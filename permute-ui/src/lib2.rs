// use neon::context::*;
// use neon::prelude::*;
// use once_cell::sync::Lazy;
// use permute::permute_files::*;
// use permute::process::*;
// use std::sync::mpsc;
// use std::sync::Mutex;
// use std::thread;
// use std::time::Duration;

// #[derive(Debug, Clone)]
// struct SharedState {
//     pub files: Vec<String>,
//     pub output: String,
//     pub input_trail: f64,
//     pub output_trail: f64,
//     pub permutations: usize,
//     pub permutation_depth: usize,
//     pub processor_pool: Vec<PermuteNodeName>,
//     pub normalise_at_end: bool,
//     pub high_sample_rate: bool,
//     pub processor_count: Option<i32>,

//     pub update_permute_node_progress: UpdatePermuteNodeProgress,
//     pub update_set_processors: UpdateSetProcessors,

//     pub processing_progress: f64,
//     pub processing_permutation: String,
//     pub send_channel: Channel,
// }

// impl SharedState {
//     fn to_permute_params(&self) -> PermuteFilesParams {
//         PermuteFilesParams {
//             files: self.files.clone(),
//             high_sample_rate: self.high_sample_rate,
//             input_trail: self.input_trail,
//             normalise_at_end: self.normalise_at_end,
//             output: self.output.clone(),
//             output_trail: self.output_trail,
//             permutation_depth: self.permutation_depth,
//             permutations: self.permutations,
//             processor_count: self.processor_count,
//             processor_pool: self.processor_pool.clone(),
//             update_permute_node_progress: self.update_permute_node_progress,
//             update_set_processors: self.update_set_processors,
//         }
//     }
// }

// impl Finalize for SharedState {}

// static SHARED_STATE: Lazy<Mutex<SharedState>> = Lazy::new(|| {
//     Mutex::new(SharedState {
//         files: vec![],
//         high_sample_rate: false,
//         input_trail: 2.0,
//         normalise_at_end: true,
//         output: String::from(
//             "/Users/jonnywildey/rustcode/permute/permute-core/renders/vibebeepui.wav",
//         ),
//         output_trail: 0.0,
//         permutation_depth: 1,
//         permutations: 3,
//         processor_count: None,
//         update_permute_node_progress: |_, _, _| {},
//         update_set_processors: |_, _| {},
//         processor_pool: vec![
//             PermuteNodeName::Reverse,
//             PermuteNodeName::MetallicDelay,
//             PermuteNodeName::RhythmicDelay,
//             PermuteNodeName::HalfSpeed,
//             PermuteNodeName::DoubleSpeed,
//             PermuteNodeName::Wow,
//             PermuteNodeName::Flutter,
//             PermuteNodeName::Chorus,
//         ],

//         processing_permutation: String::default(),
//         processing_progress: 0.0,
//         send_channel: None,
//     })
// });

// // static STATE_CALLBACK: Lazy<Mutex<(Box<dyn Fn(_) -> _>)>> =
// //     Lazy::new(|| Mutex::new(Box::new(|| {})));

// fn add_file(file: String) {
//     let mut state = SHARED_STATE.lock().unwrap();
//     state.files.push(file);
// }

// fn get_state() -> SharedState {
//     SHARED_STATE.lock().unwrap().clone()
// }

// // fn update(tx: mpsc::Sender<SharedStateType>) {
// //     thread::spawn(move || {
// //         for _ in 1..10 {
// //             add_file();
// //             let value = get_state();
// //             tx.send(value).unwrap();
// //             // callback.call(&mut cx, cx.undefined(), [cx.number(value)]);
// //             thread::sleep(Duration::from_millis(1000));
// //         }
// //     });
// // }

// fn js_get_state(mut cx: FunctionContext) -> JsResult<JsBox<SharedState>> {
//     let state = get_state();
//     let state = cx.boxed(state.clone());

//     Ok(state)
// }

// fn js_add_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let file = cx.argument::<JsString>(0)?.value(&mut cx);
//     add_file(file);
//     Ok(cx.undefined())
// }

// fn js_register_updates(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     println!("here");
//     let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
//     let undefined = cx.undefined();
//     let mut state = SHARED_STATE.lock().unwrap();

//     let mut channel = cx.channel();
//     state.send_channel = channel;

//     // *state_callback = Box::new(|| {
//     //     callback.call(&mut cx, undefined, [state]).unwrap();
//     // });

//     let cb = || {};

//     thread::spawn(move || {
//         let mut state = SHARED_STATE.lock().unwrap();
//         // Send a closure as a task to be executed by the JavaScript event
//         // loop. This _will_ block the event loop while executing.
//         channel.send(move |mut cx| {
//             let callback = callback.into_inner(&mut cx);
//             let this = cx.undefined();
//             let state = cx.boxed(get_state());
//             let args = vec![state];
//             callback.call(&mut cx, this, args)?;

//             Ok(())
//         });
//     });

//     println!("here2");

//     // for received in rx {
//     //     println!("cbd");
//     //     cb();

//     //     thread::sleep(Duration::from_millis(50));
//     // }

//     println!("here3");

//     Ok(undefined)
// }

// fn js_run_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
//     let undefined = cx.undefined();

//     let handle = thread::spawn(move || {
//         let mut state = SHARED_STATE.lock().unwrap();

//         fn update_set_processors(permutation: Permutation, processors: Vec<PermuteNodeName>) {
//             let pretty_processors = processors
//                 .iter()
//                 .map(|p| get_processor_display_name(*p))
//                 .collect::<Vec<String>>();

//             // state.send_channel.send(1.0).unwrap();

//             // println!(
//             //     "Permutating {} Processors {:#?}",
//             //     permutation.output, pretty_processors
//             // );

//             let mut lock = SHARED_STATE.try_lock();
//             if let Ok(ref mut mutex) = lock {
//                 mutex.processing_permutation = permutation.output.clone();
//                 println!("here")
//             } else {
//                 println!("try_lock failed");
//             }
//         }

//         fn update_permute_node_progress(
//             permutation: Permutation,
//             _: PermuteNodeName,
//             event: PermuteNodeEvent,
//         ) {
//             match event {
//                 PermuteNodeEvent::NodeProcessStarted => {}
//                 PermuteNodeEvent::NodeProcessComplete => {
//                     let percentage_progress: f64 = ((permutation.node_index as f64 + 1.0)
//                         / permutation.processors.len() as f64)
//                         * 100.0;
//                     // let mut state = SHARED_STATE.lock().unwrap();
//                     // state.processing_progress = percentage_progress.round();
//                     // println!("{}%", percentage_progress.round())
//                 }
//             }
//         }

//         state.update_set_processors = update_set_processors;
//         state.update_permute_node_progress = update_permute_node_progress;

//         println!("permuting");
//         permute_files(state.to_permute_params());
//         println!("permuted")
//     });

//     handle.join();

//     Ok(undefined)
// }

// #[neon::main]
// fn main(mut cx: ModuleContext) -> NeonResult<()> {
//     // thread::spawn(|| {
//     //     for _ in 1..10 {
//     //         increment();
//     //         thread::sleep(Duration::from_millis(1000));
//     //     }
//     // });
//     let c = cx.channel();

//     cx.export_function("addFile", js_add_file)?;
//     cx.export_function("getState", js_get_state)?;
//     cx.export_function("registerUpdates", js_register_updates)?;
//     cx.export_function("runProcess", js_run_process)?;

//     Ok(())
// }
