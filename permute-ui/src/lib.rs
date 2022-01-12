use neon::context::*;
use neon::prelude::*;
use once_cell::sync::Lazy;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
// use std::error::Error;
// use std::sync::mpsc;
// use std::thread;

static COUNTER: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));

fn increment() {
    let mut counter = COUNTER.lock().unwrap();
    *counter += 1;
}

fn get_value() -> i32 {
    *COUNTER.lock().unwrap()
}

fn update(tx: mpsc::Sender<i32>) {
    thread::spawn(move || {
        for _ in 1..10 {
            increment();
            let value = get_value();
            tx.send(value).unwrap();
            // callback.call(&mut cx, cx.undefined(), [cx.number(value)]);
            thread::sleep(Duration::from_millis(1000));
        }
    });
}

fn js_get_value(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let value = get_value();
    Ok(cx.number(value))
}

fn js_increment(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    increment();
    Ok(cx.undefined())
}

fn js_register_updates(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    let undefined = cx.undefined();
    let (tx, rx) = mpsc::channel();

    update(tx);

    for received in rx {
        let undefined = cx.undefined();
        let number = cx.number(received);
        callback.call(&mut cx, undefined, [number])?;
    }

    Ok(undefined)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    // thread::spawn(|| {
    //     for _ in 1..10 {
    //         increment();
    //         thread::sleep(Duration::from_millis(1000));
    //     }
    // });

    cx.export_function("increment", js_increment)?;
    cx.export_function("getValue", js_get_value)?;
    cx.export_function("registerUpdates", js_register_updates)?;

    Ok(())
}
