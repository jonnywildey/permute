mod sharedstate;

use neon::prelude::*;
use permute::permute_files::*;
use sharedstate::*;
use std::fmt::Error;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type ProcessorCallback = Box<dyn FnOnce(&Channel, SharedState) + Send>;

// Wraps a SQLite connection a channel, allowing concurrent access
struct Processor {
    tx: mpsc::Sender<ProcessorMessage>,
}

// Messages sent on the database channel
enum ProcessorMessage {
    Run(ProcessorCallback),
    AddFile(String),
    GetStateCallback(ProcessorCallback),
    Cancel,
}

impl Finalize for Processor {}

// Internal implementation
impl Processor {
    // Creates a new instance of `Processor`
    //
    // 1. Creates a js/processor channel
    // 2. Creates a permute/processor channel
    // 3. Spawns a thread and moves the channel receiver and connection to it
    // 4. On a separate thread, read closures off the channel and execute with access
    //    to the connection.
    fn new<'a, C>(cx: &mut C) -> Result<Self, Error>
    where
        C: Context<'a>,
    {
        // Channel for sending callbacks to execute on the processor connection thread
        let (tx, rx) = mpsc::channel::<ProcessorMessage>();
        let channel = cx.channel();

        // process
        let (permute_tx, permute_rx) = mpsc::channel::<PermuteUpdate>();
        let state = Arc::new(Mutex::new(SharedState::init(permute_tx)));

        // process thread
        let js_state = Arc::clone(&state);
        thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                let mut state = js_state.lock().unwrap();
                match message {
                    ProcessorMessage::GetStateCallback(f) => {
                        f(&channel, state.clone());
                    }
                    ProcessorMessage::Run(f) => {
                        state.run_process();
                        f(&channel, state.clone());
                    }
                    ProcessorMessage::AddFile(file) => {
                        state.add_file(file);
                    }
                    ProcessorMessage::Cancel => break,
                }
            }
        });

        let process_state = Arc::clone(&state);

        // // processor/shared state updates thread.
        thread::spawn(move || {
            while let Ok(message) = permute_rx.recv() {
                let mut state = process_state.lock().unwrap();
                match message {
                    PermuteUpdate::UpdatePermuteNodeCompleted(permutation, _, _) => {
                        state.update_output_progress(permutation);
                    }
                    PermuteUpdate::UpdatePermuteNodeStarted(_, _, _) => {}
                    PermuteUpdate::UpdateSetProcessors(permutation, processors) => {
                        state.add_output_progress(permutation, processors);
                        // let pretty_processors = processors
                        //     .iter()
                        //     .map(|p| get_processor_display_name(*p))
                        //     .collect::<Vec<String>>();
                        // println!(
                        //     "Permutating {} Processors {:#?}",
                        //     permutation.output, pretty_processors
                        // );
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    fn cancel(&self) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::Cancel)
    }

    fn set_state_callback(
        &self,
        callback: impl FnOnce(&Channel, SharedState) + Send + 'static,
    ) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx
            .send(ProcessorMessage::GetStateCallback(Box::new(callback)))
    }

    fn run(
        &self,
        callback: impl FnOnce(&Channel, SharedState) + Send + 'static,
    ) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::Run(Box::new(callback)))
    }

    fn add_file(&self, file: String) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::AddFile(file))
    }
}

// Methods exposed to JavaScript
impl Processor {
    // Create a new instance of `Processor` and place it inside a `JsBox`
    // JavaScript can hold a reference to a `JsBox`, but the contents are opaque
    fn js_init(mut cx: FunctionContext) -> JsResult<JsBox<Processor>> {
        let processor = Processor::new(&mut cx).or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.boxed(processor))
    }

    fn js_cancel(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        // Get the `this` value as a `JsBox<Database>`
        cx.this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?
            .cancel()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    fn js_get_state_callback(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

        let processor = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        processor
            .set_state_callback(move |channel, state| {
                channel.send(move |mut cx| {
                    let callback = callback.into_inner(&mut cx);
                    let this = cx.undefined();

                    let js_state = format!("{:#?}", state);
                    let args = vec![cx.string(js_state)];

                    callback.call(&mut cx, this, args)?;

                    Ok(())
                });
            })
            .or_else(|err| cx.throw_error(err.to_string()))
            .unwrap();

        Ok(cx.undefined())
    }

    // Run process
    fn js_run_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);

        let processor = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        processor
            .run(move |channel, state| {
                channel.send(move |mut cx| {
                    let callback = callback.into_inner(&mut cx);
                    let this = cx.undefined();

                    let js_state = format!("{:#?}", state);
                    let args = vec![cx.string(js_state)];

                    callback.call(&mut cx, this, args)?;

                    Ok(())
                });
            })
            .or_else(|err| cx.throw_error(err.to_string()))
            .unwrap();

        Ok(cx.undefined())
    }

    fn js_add_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);

        let processor = cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut cx)?;

        processor
            .add_file(file)
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("init", Processor::js_init)?;
    cx.export_function("cancel", Processor::js_cancel)?;
    cx.export_function("getStateCallback", Processor::js_get_state_callback)?;
    cx.export_function("runProcess", Processor::js_run_process)?;
    cx.export_function("addFile", Processor::js_add_file)?;

    Ok(())
}
