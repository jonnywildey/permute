const {
  init, cancel, runProcess, addFile, addProcessor, removeProcessor,
  getStateCallback, setOutput, setDepth, setInputTrail, getSingleProcessCallback,
  setOutputTrail, setPermutations, setNormalised, removeFile, reverseFile,
  saveSettings, loadSettings,
} = require("../permute-library");

const PERMUTE_POLL_LATENCY = 100;

export interface IPermuteState {
  output: string,
  error: string,
  processing: boolean,
  highSampleRate: boolean,
  inputTrail: number,
  outputTrail: 0,
  files: IPermutationInput[],
  permutations: number,
  permutationDepth: number,
  processorCount: number,
  processorPool: string[],
  allProcessors: string[],
  normaliseAtEnd: boolean,
  permutationOutputs: IPermutationOutput[];
};

export interface IPermutationInput {
  path: string;
  name: string;
  durationSec: number;
  image: string;
}

export interface IPermutationOutput {
  path: string;
  progress: number;
  image: string;
  processors: string[];
  name: string;
  durationSec: number;
}

export type GetStateCallback = (state: IPermuteState) => void;

/**
 * Wrapper for the boxed `Processor`
 */
export function createPermuteProcessor() {
  const permuteLibrary = init();
  let pollHandle: NodeJS.Timer | undefined = undefined;

  const getStateCb = (cb: GetStateCallback) => {
    return getStateCallback.call(permuteLibrary, cb);
  };


  return {
    cancel() {
      cancel.call(permuteLibrary);
    },
    runProcess(updateFn: GetStateCallback, onFinished: GetStateCallback) {
      pollHandle = setInterval(() => {
        getStateCallback.call(permuteLibrary, (state: IPermuteState) => {
          if (!state.processing) {
            clearInterval(pollHandle!);
            return onFinished(state);
          }
          updateFn(state);
        });
      }, PERMUTE_POLL_LATENCY);
      return runProcess.call(permuteLibrary);
    },
    addFile(file: string) {
      return addFile.call(permuteLibrary, file);
    },
    removeFile(file: string) {
      return removeFile.call(permuteLibrary, file);
    },
    reverseFile(file: string, updateFn: GetStateCallback, onFinished: GetStateCallback) {
      pollHandle = setInterval(() => {
        getStateCallback.call(permuteLibrary, (state: IPermuteState) => {
          if (!state.processing) {
            clearInterval(pollHandle!);
            return onFinished(state);
          }
          updateFn(state);
        });
      }, PERMUTE_POLL_LATENCY);
      return reverseFile.call(permuteLibrary, file);
    },
    addProcessor(name: string) {
      return addProcessor.call(permuteLibrary, name);
    },
    removeProcessor(name: string) {
      return removeProcessor.call(permuteLibrary, name);
    },
    setOutput(output: string) {
      return setOutput.call(permuteLibrary, output);
    },
    setDepth(output: string) {
      return setDepth.call(permuteLibrary, output);
    },
    setPermutations(output: string) {
      return setPermutations.call(permuteLibrary, output);
    },
    setNormalised(output: string) {
      return setNormalised.call(permuteLibrary, output);
    },
    setInputTrail(output: string) {
      return setInputTrail.call(permuteLibrary, output);
    },
    setOutputTrail(output: string) {
      return setOutputTrail.call(permuteLibrary, output);
    },
    loadSettings(file: string) {
      return loadSettings.call(permuteLibrary, file);
    },
    saveSettings(file: string) {
      return saveSettings.call(permuteLibrary, file);
    },
    getStateCallback,
    async getState(): Promise<IPermuteState> {
      return new Promise(res => getStateCb((state) => {
        res(state);
      }))
    }
  }
}

