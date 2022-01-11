use neon::prelude::*;
use permute::permute_files::*;
use permute::process::*;

#[derive(Debug)]
struct ApplicationState {
    pub permute_params: PermuteFilesParams,
    pub progress: i32,
}

impl Finalize for ApplicationState {}

impl ApplicationState {
    fn default() -> ApplicationState {
        fn update_set_processors(permutation: Permutation, names: Vec<PermuteNodeName>) {}
        fn update_permute_node_progress(
            permutation: Permutation,
            name: PermuteNodeName,
            event: PermuteNodeEvent,
        ) {
        }
        let processor_pool: Vec<PermuteNodeName> = vec![
            PermuteNodeName::Reverse,
            PermuteNodeName::MetallicDelay,
            PermuteNodeName::RhythmicDelay,
            PermuteNodeName::HalfSpeed,
            PermuteNodeName::DoubleSpeed,
            PermuteNodeName::Wow,
            PermuteNodeName::Flutter,
            PermuteNodeName::Chorus,
        ];
        let permute_params = PermuteFilesParams {
            files: vec![String::from(
                "/Users/jonnywildey/rustcode/permute/permute-core/examples/vibe.wav",
            )],
            high_sample_rate: false,
            input_trail: 2_f64,
            normalise_at_end: true,
            output: String::from(
                "/Users/jonnywildey/rustcode/permute/permute-core/renders/vibeui.wav",
            ),
            output_trail: 2_f64,
            permutation_depth: 1,
            permutations: 3,
            processor_count: None,
            processor_pool,
            update_set_processors,
            update_permute_node_progress,
        };

        let progress = 0;

        Self {
            permute_params,
            progress,
        }
    }
}

impl ApplicationState {
    fn init_js(mut cx: FunctionContext) -> JsResult<JsBox<ApplicationState>> {
        let state = ApplicationState::default();

        let boxed = cx.boxed(state);
        Ok(boxed)
    }

    fn get_state_js(mut cx: FunctionContext) -> JsResult<JsObject> {
        let state = cx.argument::<JsBox<ApplicationState>>(0)?;
        let state = state.downcast_or_throw::<JsBox<ApplicationState>, _>(&mut cx)?;
        println!("{:#?}", state.permute_params);
        let output = cx.string(state.permute_params.output.to_owned());

        let js_permute_params: Handle<JsObject> = cx.empty_object();
        js_permute_params.set(&mut cx, "output", output)?;

        Ok(js_permute_params)
    }

    fn run_js(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let state = cx.argument::<JsBox<ApplicationState>>(0)?;
        let state = state.downcast_or_throw::<JsBox<ApplicationState>, _>(&mut cx)?;
        permute_files(state.permute_params.clone());
        Ok(cx.undefined())
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("init", ApplicationState::init_js)?;
    cx.export_function("getState", ApplicationState::get_state_js)?;
    cx.export_function("run", ApplicationState::run_js)?;

    Ok(())
}
