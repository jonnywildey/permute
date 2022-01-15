const { init, cancel, runProcess, addFile, getStateCallback } = require("../permute-library");

const PERMUTE_POLL_LATENCY = 50;

export type PermuteState = any;

export type GetStateCallback = (state: PermuteState) => void;

// Wrapper class for the boxed `Processor` for idiomatic JavaScript usage
class PermuteProcessor {
  private permuteLibrary: any;
  private pollHandle?: NodeJS.Timer;


  constructor() {
    this.permuteLibrary = init();
    this.pollHandle = undefined;
  }

  cancel() {
    cancel.call(this.permuteLibrary);
  }

  pollForStateUpdates(cb: GetStateCallback) {
    this.pollHandle = setInterval(() => { });
  }

  runProcess(cb: GetStateCallback) {
    this.pollHandle = setInterval(() => {
      getStateCallback.call(this.permuteLibrary, cb);
    });
    return runProcess.call(this.permuteLibrary, (state: PermuteState) => {
      clearInterval(this.pollHandle!);
      cb(state);
    }, PERMUTE_POLL_LATENCY);
  }

  addFile(file: string) {
    return addFile.call(this.permuteLibrary, file);
  }

  getStateCallback(cb: GetStateCallback) {
    return getStateCallback.call(this.permuteLibrary, cb);
  }
}

const run = async () => {
  const processor = new PermuteProcessor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.getStateCallback((state) => {
    console.log(state);
  });
  processor.runProcess((state) => {
    console.log("woo", state)
  });
}

run();


