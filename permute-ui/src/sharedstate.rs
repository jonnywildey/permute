use std::sync::mpsc;

use neon::prelude::*;
use permute::permute_files::*;
use permute::process::*;

#[derive(Debug, Clone)]
pub struct SharedState {
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

    pub update_sender: mpsc::Sender<PermuteUpdate>,
}

impl SharedState {
    pub fn init(update_sender: mpsc::Sender<PermuteUpdate>) -> Self {
        Self {
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
            update_sender,
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
        }
    }

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
            update_sender: self.update_sender.to_owned(),
        }
    }

    pub fn add_file(&mut self, file: String) {
        let _ = &self.files.push(file);
    }

    pub fn run_process(&self) {
        let permute_params = Self::to_permute_params(&self);
        permute_files(permute_params);
    }
}

impl Finalize for SharedState {}

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
