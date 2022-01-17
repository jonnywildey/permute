const { init, cancel, runProcess, addFile, getStateCallback } = require("../permute-library");

const PERMUTE_POLL_LATENCY = 50;

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
    pollForStateUpdates(cb: GetStateCallback) {
      pollHandle = setInterval(() => { });
    },
    runProcess(cb: GetStateCallback) {
      pollHandle = setInterval(() => {
        getStateCallback.call(permuteLibrary, cb);
      });
      return runProcess.call(permuteLibrary, (state: PermuteState) => {
        clearInterval(pollHandle!);
        cb(state);
      }, PERMUTE_POLL_LATENCY);
    },
    addFile(file: string) {
      return addFile.call(permuteLibrary, file);
    },
    getStateCallback(cb: GetStateCallback) {
      return getStateCallback.call(permuteLibrary, cb);
    }
  }
}

// const run = async () => {
//   const processor = createPermuteProcessor();
//   processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
//   processor.getStateCallback((state) => {
//     console.log(state);
//   });
//   processor.runProcess((state) => {
//     console.log("woo", state)
//   });
// }

// run();


