mod sharedstate;

use neon::prelude::*;
use permute::display_node::*;
use permute::permute_files::*;
use sharedstate::*;
use std::fmt::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type ProcessorCallback = Box<dyn FnOnce(&Channel, SharedState) + Send>;

// Wraps a SQLite connection a channel, allowing concurrent access
struct Processor {
    tx: mpsc::Sender<ProcessorMessage>,
}

// Messages sent on the database channel
enum ProcessorMessage {
    Run,
    AddFile(String),
    RemoveFile(String),
    ReverseFile(String),
    TrimFile(String),
    AddProcessor(String),
    RemoveProcessor(String),
    SetOutput(String),
    GetStateCallback(ProcessorCallback),
    SetNormalised(bool),
    SetPermutationDepth(usize),
    SetPermutations(usize),
    SetInputTrail(f64),
    SetOutputTrail(f64),
    LoadSettingsFromJson(String),
    SaveSettingsToJson(String),
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
        let process_state = Arc::clone(&state);

        thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                let mut state = js_state.lock().unwrap();
                match message {
                    ProcessorMessage::GetStateCallback(f) => {
                        f(&channel, state.clone());
                    }
                    // update state functions to handle and print errors AI!
                    ProcessorMessage::Run => {
                        state.run_process();
                    }
                    ProcessorMessage::AddFile(file) => {
                        state.add_file(file);
                    }
                    ProcessorMessage::RemoveFile(file) => {
                        state.remove_file(file);
                    }
                    ProcessorMessage::ReverseFile(file) => {
                        state.reverse_file(file);
                    }
                    ProcessorMessage::TrimFile(file) => {
                        state.trim_file(file);
                    }
                    ProcessorMessage::AddProcessor(name) => {
                        state.add_processor(name);
                    }
                    ProcessorMessage::RemoveProcessor(name) => {
                        state.remove_processor(name);
                    }
                    ProcessorMessage::SetOutput(output) => {
                        state.set_output(output);
                    }
                    ProcessorMessage::SetInputTrail(trail) => {
                        state.set_input_trail(trail);
                    }
                    ProcessorMessage::SetOutputTrail(trail) => {
                        state.set_output_trail(trail);
                    }
                    ProcessorMessage::SetNormalised(normalised) => {
                        state.set_normalised(normalised);
                    }
                    ProcessorMessage::SetPermutations(permutations) => {
                        state.set_permutations(permutations);
                    }
                    ProcessorMessage::SetPermutationDepth(depth) => {
                        state.set_depth(depth);
                    }
                    ProcessorMessage::LoadSettingsFromJson(file) => {
                        state.read_from_json(file).unwrap_or(())
                    }
                    ProcessorMessage::SaveSettingsToJson(file) => {
                        state.write_to_json(file).unwrap_or(())
                    }
                    ProcessorMessage::Cancel => break,
                }
            }
        });

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
                    }
                    PermuteUpdate::ProcessComplete => {
                        state.set_finished();
                    }
                    PermuteUpdate::Error(err) => {
                        state.set_finished();
                        state.set_error(err);
                    }
                }
            }
        });

        Ok(Self { tx })
    }

    fn set_state_callback(
        &self,
        callback: impl FnOnce(&Channel, SharedState) + Send + 'static,
    ) -> Result<(), mpsc::SendError<ProcessorMessage>> {
        self.tx
            .send(ProcessorMessage::GetStateCallback(Box::new(callback)))
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
        js_hook!(ProcessorMessage::Cancel, cx);
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

                    let output = cx.string(state.output.clone());
                    let error = cx.string(state.error.clone());
                    let processing = cx.boolean(state.processing);
                    let high_sample_rate = cx.boolean(state.high_sample_rate);
                    let input_trail = cx.number(state.input_trail);
                    let output_trail = cx.number(state.output_trail);
                    let permutations = cx.number(state.permutations as u32);
                    let permutation_depth = cx.number(state.permutation_depth as u32);
                    let processor_count = cx.number(state.processor_count.unwrap_or(0) as u32);
                    let normalise_at_end = cx.boolean(state.normalise_at_end);

                    let files = cx.empty_array();
                    for i in 0..state.files.len() {
                        let input_obj = cx.empty_object();
                        let path = cx.string(state.files[i].path.clone());
                        let name = cx.string(state.files[i].name.clone());
                        let image = cx.string(state.files[i].image.clone());
                        let duration_sec = cx.number(state.files[i].duration_sec);

                        input_obj.set(&mut cx, "path", path)?;
                        input_obj.set(&mut cx, "name", name)?;
                        input_obj.set(&mut cx, "image", image)?;
                        input_obj.set(&mut cx, "durationSec", duration_sec)?;

                        files.set(&mut cx, i as u32, input_obj)?;
                    }
                    let processor_pool = cx.empty_array();
                    for i in 0..state.processor_pool.len() {
                        let str = cx.string(get_processor_display_name(state.processor_pool[i]));
                        processor_pool.set(&mut cx, i as u32, str)?;
                    }
                    let all_processors = cx.empty_array();
                    for i in 0..state.all_processors.len() {
                        let str = cx.string(get_processor_display_name(state.all_processors[i]));
                        all_processors.set(&mut cx, i as u32, str)?;
                    }
                    let permutation_outputs = cx.empty_array();
                    for i in 0..state.permutation_outputs.len() {
                        let permutation_output = &state.permutation_outputs[i];
                        let output_obj = cx.empty_object();
                        let output = cx.string(permutation_output.output.clone());
                        output_obj.set(&mut cx, "path", output)?;
                        let name = cx.string(permutation_output.audio_info.name.clone());
                        output_obj.set(&mut cx, "name", name)?;
                        let image = cx.string(permutation_output.audio_info.image.clone());
                        output_obj.set(&mut cx, "image", image)?;
                        let progress = cx.number(permutation_output.progress);
                        output_obj.set(&mut cx, "progress", progress)?;
                        let duration_sec = cx.number(permutation_output.audio_info.duration_sec);
                        output_obj.set(&mut cx, "durationSec", duration_sec)?;

                        let node_names = cx.empty_array();
                        for j in 0..permutation_output.processors.len() {
                            let display_name = cx.string(get_processor_display_name(
                                permutation_output.processors[j],
                            ));
                            node_names.set(&mut cx, j as u32, display_name)?;
                        }
                        output_obj.set(&mut cx, "processors", node_names)?;
                        permutation_outputs.set(&mut cx, i as u32, output_obj)?;
                    }

                    let obj = cx.empty_object();
                    obj.set(&mut cx, "output", output)?;
                    obj.set(&mut cx, "error", error)?;
                    obj.set(&mut cx, "processing", processing)?;
                    obj.set(&mut cx, "highSampleRate", high_sample_rate)?;
                    obj.set(&mut cx, "inputTrail", input_trail)?;
                    obj.set(&mut cx, "outputTrail", output_trail)?;
                    obj.set(&mut cx, "files", files)?;
                    obj.set(&mut cx, "permutations", permutations)?;
                    obj.set(&mut cx, "permutationDepth", permutation_depth)?;
                    obj.set(&mut cx, "processorCount", processor_count)?;
                    obj.set(&mut cx, "processorPool", processor_pool)?;
                    obj.set(&mut cx, "allProcessors", all_processors)?;
                    obj.set(&mut cx, "normaliseAtEnd", normalise_at_end)?;
                    obj.set(&mut cx, "permutationOutputs", permutation_outputs)?;

                    let args = vec![obj];

                    callback.call(&mut cx, this, args)?;

                    Ok(())
                });
            })
            .or_else(|err| cx.throw_error(err.to_string()))
            .unwrap();

        Ok(cx.undefined())
    }

    fn js_run_process(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        js_hook!(ProcessorMessage::Run, cx);
        Ok(cx.undefined())
    }

    fn js_reverse_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::ReverseFile, cx);
        Ok(cx.undefined())
    }

    fn js_trim_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::TrimFile, cx);
        Ok(cx.undefined())
    }

    fn js_add_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::AddFile, cx);
        Ok(cx.undefined())
    }

    fn js_remove_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::RemoveFile, cx);
        Ok(cx.undefined())
    }

    fn js_set_permutations(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let permutations = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
        js_hook!(permutations, ProcessorMessage::SetPermutations, cx);
        Ok(cx.undefined())
    }

    fn js_set_depth(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let depth = cx.argument::<JsNumber>(0)?.value(&mut cx) as usize;
        js_hook!(depth, ProcessorMessage::SetPermutationDepth, cx);
        Ok(cx.undefined())
    }

    fn js_set_normalised(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let normalised = cx.argument::<JsBoolean>(0)?.value(&mut cx);
        js_hook!(normalised, ProcessorMessage::SetNormalised, cx);
        Ok(cx.undefined())
    }

    fn js_set_input_trail(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let input_trail = cx.argument::<JsNumber>(0)?.value(&mut cx);
        js_hook!(input_trail, ProcessorMessage::SetInputTrail, cx);
        Ok(cx.undefined())
    }

    fn js_set_output_trail(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let output_trail = cx.argument::<JsNumber>(0)?.value(&mut cx);
        js_hook!(output_trail, ProcessorMessage::SetOutputTrail, cx);
        Ok(cx.undefined())
    }

    fn js_add_processor(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(name, ProcessorMessage::AddProcessor, cx);
        Ok(cx.undefined())
    }

    fn js_remove_processor(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let name = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(name, ProcessorMessage::RemoveProcessor, cx);
        Ok(cx.undefined())
    }

    fn js_set_output(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::SetOutput, cx);
        Ok(cx.undefined())
    }

    fn js_save_settings(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::SaveSettingsToJson, cx);
        Ok(cx.undefined())
    }

    fn js_load_settings(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::LoadSettingsFromJson, cx);
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
    cx.export_function("removeFile", Processor::js_remove_file)?;
    cx.export_function("addProcessor", Processor::js_add_processor)?;
    cx.export_function("removeProcessor", Processor::js_remove_processor)?;
    cx.export_function("setOutput", Processor::js_set_output)?;
    cx.export_function("setDepth", Processor::js_set_depth)?;
    cx.export_function("setInputTrail", Processor::js_set_input_trail)?;
    cx.export_function("setOutputTrail", Processor::js_set_output_trail)?;
    cx.export_function("setPermutations", Processor::js_set_permutations)?;
    cx.export_function("setNormalised", Processor::js_set_normalised)?;
    cx.export_function("reverseFile", Processor::js_reverse_file)?;
    cx.export_function("trimFile", Processor::js_trim_file)?;
    cx.export_function("saveSettings", Processor::js_save_settings)?;
    cx.export_function("loadSettings", Processor::js_load_settings)?;

    Ok(())
}

macro_rules! js_hook {
    ($parameter:expr, $message:expr, $cx:expr) => {{
        let processor = $cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut $cx)?;

        processor
            .tx
            .send($message($parameter))
            .or_else(|err| $cx.throw_error(err.to_string()))?;
    }};
    ($message:expr, $cx:expr) => {{
        let processor = $cx
            .this()
            .downcast_or_throw::<JsBox<Processor>, _>(&mut $cx)?;

        processor
            .tx
            .send($message)
            .or_else(|err| $cx.throw_error(err.to_string()))?;
    }};
}
pub(crate) use js_hook;
