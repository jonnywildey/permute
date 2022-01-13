use neon::context::*;
use neon::prelude::*;
use once_cell::sync::Lazy;
use permute::permute_files::*;
use permute::process::*;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct SharedState {
    pub files: Vec<String>,
    pub output: String,
    pub input_trail: f64,
    pub output_trail: f64,
    pub permutations: usize,
    pub permutation_depth: usize,
    pub processor_pool: Vec<PermuteNodeName>,
    pub normalise_at_end: bool,
    pub high_sample_rate: bool,
    pub processor_count: Option<i32>,

    pub update_permute_node_progress: UpdatePermuteNodeProgress,
    pub update_set_processors: UpdateSetProcessors,
}

impl SharedState {
    fn to_permute_params(&self) -> PermuteFilesParams {
        PermuteFilesParams {
            files: self.files.clone(),
            high_sample_rate: self.high_sample_rate,
            input_trail: self.input_trail,
            normalise_at_end: self.normalise_at_end,
            output: self.output.clone(),
            output_trail: self.output_trail,
            permutation_depth: self.permutation_depth,
            permutations: self.permutations,
            processor_count: self.processor_count,
            processor_pool: self.processor_pool.clone(),
            update_permute_node_progress: self.update_permute_node_progress,
            update_set_processors: self.update_set_processors,
        }
    }
}

impl Finalize for SharedState {}

static SHARED_STATE: Lazy<Mutex<SharedState>> = Lazy::new(|| {
    Mutex::new(SharedState {
        files: vec![],
        high_sample_rate: false,
        input_trail: 2.0,
        normalise_at_end: true,
        output: String::from(
            "/Users/jonnywildey/rustcode/permute/permute-core/renders/vibebeepui.wav",
        ),
        output_trail: 0.0,
        permutation_depth: 1,
        permutations: 3,
        processor_count: None,
        update_permute_node_progress: |_, _, _| {},
        update_set_processors: |_, _| {},
        processor_pool: vec![
            PermuteNodeName::Reverse,
            PermuteNodeName::MetallicDelay,
            PermuteNodeName::RhythmicDelay,
            PermuteNodeName::HalfSpeed,
            PermuteNodeName::DoubleSpeed,
            PermuteNodeName::Wow,
            PermuteNodeName::Flutter,
            PermuteNodeName::Chorus,
        ],
    })
});

fn add_file(file: String) {
    let mut state = SHARED_STATE.lock().unwrap();
    state.files.push(file);
}

fn get_state() -> SharedState {
    SHARED_STATE.lock().unwrap().clone()
}

// fn update(tx: mpsc::Sender<SharedStateType>) {
//     thread::spawn(move || {
//         for _ in 1..10 {
//             add_file();
//             let value = get_state();
//             tx.send(value).unwrap();
//             // callback.call(&mut cx, cx.undefined(), [cx.number(value)]);
//             thread::sleep(Duration::from_millis(1000));
//         }
//     });
// }

fn js_get_state(mut cx: FunctionContext) -> JsResult<JsBox<SharedState>> {
    let state = get_state();
    let state = cx.boxed(state.clone());

    Ok(state)
}

fn js_add_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let file = cx.argument::<JsString>(0)?.value(&mut cx);
    add_file(file);
    Ok(cx.undefined())
}

fn js_register_updates(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    let undefined = cx.undefined();
    // let (tx, rx) = mpsc::channel();

    // for received in rx {
    //     let undefined = cx.undefined();
    //     let number = cx.number(received);
    //     callback.call(&mut cx, undefined, [number])?;
    // }

    Ok(undefined)
}

fn js_run_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let undefined = cx.undefined();

    thread::spawn(move || {
        let mut state = SHARED_STATE.lock().unwrap();

        fn update_set_processors(permutation: Permutation, processors: Vec<PermuteNodeName>) {
            let pretty_processors = processors
                .iter()
                .map(|p| get_processor_display_name(*p))
                .collect::<Vec<String>>();
            println!(
                "Permutating {} Processors {:#?}",
                permutation.output, pretty_processors
            );
        }

        fn update_permute_node_progress(
            permutation: Permutation,
            _: PermuteNodeName,
            event: PermuteNodeEvent,
        ) {
            match event {
                PermuteNodeEvent::NodeProcessStarted => {}
                PermuteNodeEvent::NodeProcessComplete => {
                    let percentage_progress: f64 = ((permutation.node_index as f64 + 1.0)
                        / permutation.processors.len() as f64)
                        * 100.0;
                    println!("{}%", percentage_progress.round())
                }
            }
        }

        state.update_set_processors = update_set_processors;
        state.update_permute_node_progress = update_permute_node_progress;

        permute_files(state.to_permute_params());
    });

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

    cx.export_function("addFile", js_add_file)?;
    cx.export_function("getState", js_get_state)?;
    cx.export_function("registerUpdates", js_register_updates)?;
    cx.export_function("runProcess", js_run_process)?;

    Ok(())
}
