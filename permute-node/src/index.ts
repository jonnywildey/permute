const {
  init, cancel, runProcess, addFile,
  addProcessor, removeProcessor,
  getStateCallback, setOutput } = require("../permute-library");

const PERMUTE_POLL_LATENCY = 20;

export interface IPermuteState {
  output: string,
  finished: boolean,
  highSampleRate: boolean,
  inputTrail: number,
  outputTrail: 0,
  files: string[],
  permutations: number,
  permutationDepth: number,
  processorCount: number,
  processorPool: string[],
  normaliseAtEnd: boolean,
  permutationOutputs: IPermutationOutput[];
};

export interface IPermutationOutput {
  output: string;
  progress: number;
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
    runProcess(updateFn: GetStateCallback) {
      pollHandle = setInterval(() => {
        getStateCallback.call(permuteLibrary, (state: IPermuteState) => {
          if (state.finished) {
            clearInterval(pollHandle!);
          }
          updateFn(state);
        });
      }, PERMUTE_POLL_LATENCY);
      return runProcess.call(permuteLibrary);
    },
    addFile(file: string) {
      return addFile.call(permuteLibrary, file);
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
    getStateCallback,
    async getState(): Promise<IPermuteState> {
      return new Promise(res => getStateCb((state) => {
        res(state);
      }))
    }
  }
}

