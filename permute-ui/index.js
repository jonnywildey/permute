// const permuteLibrary = require("./permute-library");

// console.log("start");
// permuteLibrary.registerUpdates((state) => {
//   console.log(state);
// });

// permuteLibrary.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");

// permuteLibrary.runProcess();

// setTimeout(() => { }, 20000);


"use strict";
const { init, cancel, runProcess, addFile, getStateCallback } = require("./permute-library");



// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class Processor {
  constructor() {
    this.permuteLibrary = init();
    this.pollHandle = undefined;
  }

  cancel() {
    cancel.call(this.permuteLibrary);
  }

  pollForStateUpdates(cb) {
    this.pollHandle = setInterval(() => { });
  }

  runProcess(cb) {
    this.pollHandle = setInterval(() => {
      getStateCallback.call(this.permuteLibrary, cb);
    });
    return runProcess.call(this.permuteLibrary, (state) => {
      clearInterval(this.pollHandle);
      cb(state);
    }, 50);
  }

  addFile(file) {
    return addFile.call(this.permuteLibrary, file);
  }

  getStateCallback(cb) {
    return getStateCallback.call(this.permuteLibrary, cb);
  }
}

const run = async () => {
  const processor = new Processor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.getStateCallback((state) => {
    console.log(state);
  });
  processor.runProcess((state) => {
    console.log("woo", state)
  });
}

run();


