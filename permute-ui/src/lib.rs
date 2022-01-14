mod sharedstate;

use neon::prelude::*;
use sharedstate::*;
use std::fmt::Error;
use std::sync::mpsc;
use std::thread;

type ProcessorCallback = Box<dyn FnOnce(&Channel) + Send>;

// Wraps a SQLite connection a channel, allowing concurrent access
struct Processor {
    tx: mpsc::Sender<ProcessorMessage>,
}

// Messages sent on the database channel
enum ProcessorMessage {
    Run(ProcessorCallback),
    AddFile(String),
    // Callback to be executed
    Callback(ProcessorCallback),
    // Indicates that the thread should be stopped and connection closed
    Cancel,
}

// Clean-up when Database is garbage collected, could go here
// but, it's not needed,
impl Finalize for Processor {}

// Internal implementation
impl Processor {
    // Creates a new instance of `Database`
    //
    // 1. Creates a channel
    // 2. Spawns a thread and moves the channel receiver and connection to it
    // 3. On a separate thread, read closures off the channel and execute with access
    //    to the connection.
    fn new<'a, C>(cx: &mut C) -> Result<Self, Error>
    where
        C: Context<'a>,
    {
        // Channel for sending callbacks to execute on the sqlite connection thread
        let (tx, rx) = mpsc::channel::<ProcessorMessage>();
        // Create an `Channel` for calling back to JavaScript. It is more efficient
        // to create a single channel and re-use it for all database callbacks.
        // The JavaScript process will not exit as long as this channel has not been
        // dropped.
        let channel = cx.channel();

        // process
        let mut state = SharedState::default();

        // Spawn a thread for processing database queries
        // This will not block the JavaScript main thread and will continue executing
        // concurrently.
        thread::spawn(move || {
            // Blocks until a callback is available
            // When the instance of `Database` is dropped, the channel will be closed
            // and `rx.recv()` will return an `Err`, ending the loop and terminating
            // the thread.
            while let Ok(message) = rx.recv() {
                match message {
                    ProcessorMessage::Callback(f) => {
                        // The connection and channel are owned by the thread, but _lent_ to
                        // the callback. The callback has exclusive access to the connection
                        // for the duration of the callback.
                        f(&channel);
                    }
                    ProcessorMessage::Run(f) => {
                        println!("start");
                        state.run_process();
                        println!("done")
                    }
                    ProcessorMessage::AddFile(file) => {
                        state.add_file(file);
                    }
                    // Immediately close the connection, even if there are pending messages
                    ProcessorMessage::Cancel => break,
                }
            }
        });

        Ok(Self { tx })
    }

    // Idiomatic rust would take an owned `self` to prevent use after close
    // However, it's not possible to prevent JavaScript from continuing to hold a closed database
    fn cancel(&self) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::Cancel)
    }

    fn send(
        &self,
        callback: impl FnOnce(&Channel) + Send + 'static,
    ) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::Callback(Box::new(callback)))
    }

    fn run(
        &self,
        callback: impl FnOnce(&Channel) + Send + 'static,
    ) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::Run(Box::new(callback)))
    }

    fn add_file(&self, file: String) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::AddFile(file))
    }
}

// Methods exposed to JavaScript
// The `JsBox` boxed `Database` is expected as the `this` value on all methods except `js_new`
impl Processor {
    // Create a new instance of `Processor` and place it inside a `JsBox`
    // JavaScript can hold a reference to a `JsBox`, but the contents are opaque
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<Processor>> {
        let db = Processor::new(&mut cx).or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.boxed(db))
    }

    // Manually close a database connection
    // After calling `close`, all other methods will fail
    // It is not necessary to call `close` since the database will be closed when the wrapping
    // `JsBox` is garbage collected. However, calling `close` allows the process to exit
    // immediately instead of waiting on garbage collection. This is useful in tests.
    fn js_cancel(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the `this` value as a `JsBox<Database>`
        cx.this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?
            .cancel()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    // Inserts a `name` into the database
    // Accepts a `name` and a `callback` as parameters
    fn js_insert(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the first argument as a `JsString` and convert to a Rust `String`
        let name = cx.argument::<JsString>(0)?.value(&mut cx);

        // Get the second argument as a `JsFunction`
        let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);

        // Get the `this` value as a `JsBox<Database>`
        let db = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        db.send(move |channel| {
            // do a thing

            channel.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                // let args: Vec<Handle<JsValue>> = match result {
                //     Ok(id) => vec![cx.null().upcast(), cx.number(id as f64).upcast()],
                //     Err(err) => vec![cx.error(err.to_string())?.upcast()],
                // };
                let args = vec![cx.undefined()];

                callback.call(&mut cx, this, args)?;

                Ok(())
            });
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        // This function does not have a return value
        Ok(cx.undefined())
    }

    // Run process
    fn js_run_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the first argument as a `JsString` and convert to a Rust `String`
        // let name = cx.argument::<JsString>(0)?.value(&mut cx);

        // Get the second argument as a `JsFunction`
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

        // Get the `this` value as a `JsBox<Database>`
        let db = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        db.run(move |channel| {
            // do a thing

            channel.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                // let args: Vec<Handle<JsValue>> = match result {
                //     Ok(id) => vec![cx.null().upcast(), cx.number(id as f64).upcast()],
                //     Err(err) => vec![cx.error(err.to_string())?.upcast()],
                // };
                let args = vec![cx.undefined()];

                callback.call(&mut cx, this, args)?;

                Ok(())
            });
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        // This function does not have a return value
        Ok(cx.undefined())
    }

    fn js_add_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the first argument as a `JsString` and convert to a Rust `String`
        let file = cx.argument::<JsString>(0)?.value(&mut cx);

        // Get the `this` value as a `JsBox<Database>`
        let processor = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        processor
            .add_file(file)
            .or_else(|err| cx.throw_error(err.to_string()))?;

        // This function does not have a return value
        Ok(cx.undefined())
    }

    // Get a `name` by `id` value
    // Accepts an `id` and callback as parameters
    fn js_get_by_id(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the first argument as a `JsNumber` and convert to an `f64`
        let id = cx.argument::<JsNumber>(0)?.value(&mut cx);

        // Get the second argument as a `JsFunction`
        let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);

        // Get the `this` value as a `JsBox<Database>`
        let db = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        db.send(move |channel| {
            // let result: Result<String, _> = conn
            //     .prepare("SELECT name FROM person WHERE id = ?")
            //     .and_then(|mut stmt| stmt.query_row(rusqlite::params![id], |row| row.get(0)));

            channel.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                // let args: Vec<Handle<JsValue>> = match result {
                //     // Convert the name to a `JsString` on success and upcast to a `JsValue`
                //     Ok(name) => vec![cx.null().upcast(), cx.string(name).upcast()],

                //     // If the row was not found, return `undefined` as a success instead
                //     // of throwing an exception
                //     Err(rusqlite::Error::QueryReturnedNoRows) => {
                //         vec![cx.null().upcast(), cx.undefined().upcast()]
                //     }

                //     // Convert the error to a JavaScript exception on failure
                //     Err(err) => vec![cx.error(err.to_string())?.upcast()],
                // };
                let args = vec![cx.undefined()];

                callback.call(&mut cx, this, args)?;

                Ok(())
            });
        })
        .or_else(|err| cx.throw_error(err.to_string()))?;

        // This function does not have a return value
        Ok(cx.undefined())
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("databaseNew", Processor::js_new)?;
    cx.export_function("databaseClose", Processor::js_cancel)?;
    cx.export_function("databaseInsert", Processor::js_insert)?;
    cx.export_function("databaseGetById", Processor::js_get_by_id)?;
    cx.export_function("runProcess", Processor::js_run_process)?;
    cx.export_function("addFile", Processor::js_add_file)?;

    Ok(())
}
