mod sharedstate;

use neon::prelude::*;
use permute::display_node::*;
use permute::permute_files::*;
use sharedstate::*;
use std::fmt::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::panic::catch_unwind;
use crossbeam_channel::{Sender, Receiver, SendError};

type ProcessorCallback = Box<dyn FnOnce(&Channel, SharedState) + Send>;

// Wraps a SQLite connection a channel, allowing concurrent access
struct Processor {
    tx: Sender<ProcessorMessage>,
    worker_count: usize,
}

// Messages sent on the database channel
enum ProcessorMessage {
    Run,
    AddFile(String),
    RemoveFile(String),
    DeleteOutputFile(String),
    DeleteAllOutputFiles,
    ReverseFile(String),
    TrimFile(String),
    AddProcessor(String),
    RemoveProcessor(String),
    SetOutput(String),
    GetStateCallback(ProcessorCallback),
    SetNormalised(bool),
    SetTrimAll(bool),
    SetPermutationDepth(usize),
    SetPermutations(usize),
    SetInputTrail(f64),
    SetOutputTrail(f64),
    LoadSettingsFromJson(String),
    SaveSettingsToJson(String),
    SetCreateSubdirectories(bool),
    SelectAllProcessors,
    DeselectAllProcessors,
    SetViewedWelcome(bool),
    SetMaxStretch(f64),
    Cancel,
}

impl Finalize for Processor {}

// Internal implementation
impl Processor {
    // Creates a new instance of `Processor`
    fn new<'a, C>(cx: &mut C) -> Result<Self, Error>
    where
        C: Context<'a>,
    {
        // Get number of CPU cores, but cap at 4 to avoid excessive resource usage
        let worker_count = std::thread::available_parallelism()
            .map(|n| std::cmp::min(n.get(), 4))
            .unwrap_or(1);

        let (tx, rx) = crossbeam_channel::bounded::<ProcessorMessage>(100);
        let channel = cx.channel();

        let (permute_tx, permute_rx) = crossbeam_channel::bounded::<PermuteUpdate>(100);
        let state = Arc::new(Mutex::new(SharedState::init(permute_tx)));

        let js_state = Arc::clone(&state);
        let process_state = Arc::clone(&state);
        let channel_clone = channel.clone();

        // Spawn worker threads
        for i in 0..worker_count {
            let worker_state = Arc::clone(&state);
            let worker_rx = rx.clone();
            let worker_channel = channel.clone();
            let worker_js_state = Arc::clone(&js_state);
            
            thread::Builder::new()
                .name(format!("ProcessThread_{}", i))
                .spawn(move || {
                    let result = catch_unwind(|| {
                        while let Ok(message) = worker_rx.recv() {
                            let mut state = worker_state.lock().unwrap();
                            match message {
                                ProcessorMessage::GetStateCallback(f) => {
                                    f(&worker_channel, state.clone());
                                }
                                ProcessorMessage::Run => {
                                    state.run_process();
                                }
                                ProcessorMessage::AddFile(file) => match state.add_file(file) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error adding file: {:?}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                },
                                ProcessorMessage::RemoveFile(file) => {
                                    state.remove_file(file);
                                }
                                ProcessorMessage::DeleteOutputFile(file) => {
                                    match state.delete_output_file(file) {
                                        Ok(()) => {}
                                        Err(err) => {
                                            println!("Error deleting output file: {}", err.to_string());
                                            state.set_error(err.to_string());
                                        }
                                    }
                                }
                                ProcessorMessage::DeleteAllOutputFiles => {
                                    match state.delete_all_output_files() {
                                        Ok(()) => {}
                                        Err(err) => {
                                            println!("Error deleting all output files: {}", err.to_string());
                                            state.set_error(err.to_string());
                                        }
                                    }
                                }
                                ProcessorMessage::ReverseFile(file) => match state.reverse_file(file) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error reversing file: {}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                },
                                ProcessorMessage::TrimFile(file) => match state.trim_file(file) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error trimming file: {}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                },
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
                                ProcessorMessage::SetTrimAll(trim_all) => {
                                    state.set_trim_all(trim_all);
                                }
                                ProcessorMessage::SetMaxStretch(max_stretch) => {
                                    state.set_max_stretch(max_stretch);
                                }
                                ProcessorMessage::SetPermutations(permutations) => {
                                    state.set_permutations(permutations);
                                }
                                ProcessorMessage::SetPermutationDepth(depth) => {
                                    state.set_depth(depth);
                                }
                                ProcessorMessage::LoadSettingsFromJson(file) => {
                                  match state.read_from_json(file) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error loading settings: {}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                  }
                                }
                                ProcessorMessage::SaveSettingsToJson(file) => {
                                    state.write_to_json(file).unwrap_or(())
                                }
                                ProcessorMessage::SetCreateSubdirectories(create) => {
                                    state.set_create_subdirectories(create);
                                }
                                ProcessorMessage::SelectAllProcessors => {
                                    state.select_all_processors();
                                }
                                ProcessorMessage::DeselectAllProcessors => {
                                    state.deselect_all_processors();
                                }
                                ProcessorMessage::SetViewedWelcome(viewed) => {
                                    state.set_viewed_welcome(viewed);
                                }
                                ProcessorMessage::Cancel => {
                                    state.cancel();
                                }
                            }
                        }
                    });

                    // Handle any panics in the processing thread
                    if let Err(panic) = result {
                        let mut state = worker_js_state.lock().unwrap();
                        state.set_error(format!("Processing thread panicked: {:?}", panic));
                        state.set_finished().unwrap_or_else(|e| {
                            println!("Error setting finished state after panic: {}", e);
                        });
                    }
                })
                .expect("Failed to spawn process thread");
        }

        // Spawn the update handling thread with proper cleanup
        let _update_handle = thread::Builder::new()
            .name("UpdateThread".to_string())
            .spawn(move || {
                let result = catch_unwind(|| {
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
                            PermuteUpdate::AudioInfoGenerated(file, audio_info) => {
                                state.update_output_audioinfo(file, audio_info);
                            }
                            PermuteUpdate::ProcessComplete(_) => {
                                match state.set_finished() {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error setting finished: {}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                }
                            }
                            PermuteUpdate::Error(err) => {
                                match state.set_finished() {
                                    Ok(()) => {}
                                    Err(err) => {
                                        println!("Error setting finished: {}", err.to_string());
                                        state.set_error(err.to_string());
                                    }
                                }
                                state.set_error(err);
                            }
                        }
                    }
                });

                // Handle any panics in the update thread
                if let Err(panic) = result {
                    let mut state = process_state.lock().unwrap();
                    state.set_error(format!("Update thread panicked: {:?}", panic));
                    state.set_finished().unwrap_or_else(|e| {
                        println!("Error setting finished state after panic: {}", e);
                    });
                }
            })
            .expect("Failed to spawn update thread");

        Ok(Self { tx, worker_count })
    }

    fn set_state_callback(
        &self,
        callback: impl FnOnce(&Channel, SharedState) + Send + 'static,
    ) -> Result<(), SendError<ProcessorMessage>> {
        self.tx.send(ProcessorMessage::GetStateCallback(Box::new(callback)))
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
                    let normalise_at_end: Handle<'_, JsBoolean> =
                        cx.boolean(state.normalise_at_end);
                    let trim_all: Handle<'_, JsBoolean> = cx.boolean(state.trim_all);
                    let create_subdirectories: Handle<'_, JsBoolean> =
                        cx.boolean(state.create_subdirectories);
                    let viewed_welcome: Handle<'_, JsBoolean> = cx.boolean(state.viewed_welcome);
                    let max_stretch = cx.number(state.max_stretch);

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
                    let ordered_outputs = state.get_ordered_outputs();
                    for (i, output) in ordered_outputs.iter().enumerate() {
                        let output_obj = cx.empty_object();
                        let output_path = cx.string(output.output.clone());
                        output_obj.set(&mut cx, "path", output_path)?;
                        let name = cx.string(output.audio_info.name.clone());
                        output_obj.set(&mut cx, "name", name)?;
                        let image = cx.string(output.audio_info.image.clone());
                        output_obj.set(&mut cx, "image", image)?;
                        let progress = cx.number(output.progress);
                        output_obj.set(&mut cx, "progress", progress)?;
                        let duration_sec = cx.number(output.audio_info.duration_sec);
                        output_obj.set(&mut cx, "durationSec", duration_sec)?;
                        let deleted = cx.boolean(output.deleted);
                        output_obj.set(&mut cx, "deleted", deleted)?;

                        let processors_array = cx.empty_array();
                        for (j, processor) in output.permutation.processors.iter().enumerate() {
                            let processor_obj = cx.empty_object();
                            let display_name = cx.string(get_processor_display_name(processor.name));
                            processor_obj.set(&mut cx, "name", display_name)?;
                            
                            let attributes = cx.empty_array();
                            for (k, attr) in processor.attributes.iter().enumerate() {
                                let attr_obj = cx.empty_object();
                                let key = cx.string(attr.key.clone());
                                let value = cx.string(attr.value.clone());
                                attr_obj.set(&mut cx, "key", key)?;
                                attr_obj.set(&mut cx, "value", value)?;
                                attributes.set(&mut cx, k as u32, attr_obj)?;
                            }
                            processor_obj.set(&mut cx, "attributes", attributes)?;
                            
                            processors_array.set(&mut cx, j as u32, processor_obj)?;
                        }
                        output_obj.set(&mut cx, "processors", processors_array)?;
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
                    obj.set(&mut cx, "trimAll", trim_all)?;
                    obj.set(&mut cx, "permutationOutputs", permutation_outputs)?;
                    obj.set(&mut cx, "createSubdirectories", create_subdirectories)?;
                    obj.set(&mut cx, "viewedWelcome", viewed_welcome)?;
                    obj.set(&mut cx, "maxStretch", max_stretch)?;

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

    fn js_delete_output_file(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let file = cx.argument::<JsString>(0)?.value(&mut cx);
        js_hook!(file, ProcessorMessage::DeleteOutputFile, cx);
        Ok(cx.undefined())
    }

    fn js_delete_all_output_files(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        js_hook!(ProcessorMessage::DeleteAllOutputFiles, cx);
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

    fn js_set_trim_all(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let trim_all = cx.argument::<JsBoolean>(0)?.value(&mut cx);
        js_hook!(trim_all, ProcessorMessage::SetTrimAll, cx);
        Ok(cx.undefined())
    }

    fn js_set_max_stretch(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let max_stretch = cx.argument::<JsNumber>(0)?.value(&mut cx);
        js_hook!(max_stretch, ProcessorMessage::SetMaxStretch, cx);
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

    fn js_set_create_subdirectories(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let create = cx.argument::<JsBoolean>(0)?.value(&mut cx);
        js_hook!(create, ProcessorMessage::SetCreateSubdirectories, cx);
        Ok(cx.undefined())
    }

    fn js_select_all_processors(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        js_hook!(ProcessorMessage::SelectAllProcessors, cx);
        Ok(cx.undefined())
    }

    fn js_deselect_all_processors(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        js_hook!(ProcessorMessage::DeselectAllProcessors, cx);
        Ok(cx.undefined())
    }

    fn js_set_viewed_welcome(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let viewed = cx.argument::<JsBoolean>(0)?.value(&mut cx);
        js_hook!(viewed, ProcessorMessage::SetViewedWelcome, cx);
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
    cx.export_function("deleteOutputFile", Processor::js_delete_output_file)?;
    cx.export_function("deleteAllOutputFiles", Processor::js_delete_all_output_files)?;
    cx.export_function("addProcessor", Processor::js_add_processor)?;
    cx.export_function("removeProcessor", Processor::js_remove_processor)?;
    cx.export_function("setOutput", Processor::js_set_output)?;
    cx.export_function("setDepth", Processor::js_set_depth)?;
    cx.export_function("setInputTrail", Processor::js_set_input_trail)?;
    cx.export_function("setOutputTrail", Processor::js_set_output_trail)?;
    cx.export_function("setPermutations", Processor::js_set_permutations)?;
    cx.export_function("setNormalised", Processor::js_set_normalised)?;
    cx.export_function("setTrimAll", Processor::js_set_trim_all)?;
    cx.export_function("reverseFile", Processor::js_reverse_file)?;
    cx.export_function("trimFile", Processor::js_trim_file)?;
    cx.export_function("setMaxStretch", Processor::js_set_max_stretch)?;
    cx.export_function("saveSettings", Processor::js_save_settings)?;
    cx.export_function("loadSettings", Processor::js_load_settings)?;
    cx.export_function("setCreateSubdirectories", Processor::js_set_create_subdirectories)?;
    cx.export_function("selectAllProcessors", Processor::js_select_all_processors)?;
    cx.export_function("deselectAllProcessors", Processor::js_deselect_all_processors)?;
    cx.export_function("setViewedWelcome", Processor::js_set_viewed_welcome)?;

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
