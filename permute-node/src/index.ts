const { init, cancel, runProcess, addFile, getStateCallback } = require("../permute-library");

const PERMUTE_POLL_LATENCY = 200;

export type PermuteState = any;

export type GetStateCallback = (state: PermuteState) => void;

/**
 * Wrapper for the boxed `Processor`
 */
export function createPermuteProcessor() {
  const permuteLibrary = init();
  let pollHandle: NodeJS.Timer | undefined = undefined;

  return {
    cancel() {
      cancel.call(permuteLibrary);
    },
    runProcess(updateFn: GetStateCallback) {
      pollHandle = setInterval(() => {
        getStateCallback.call(permuteLibrary, (state: PermuteState) => {
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
    getStateCallback(cb: GetStateCallback) {
      return getStateCallback.call(permuteLibrary, cb);
    }
  }
}

