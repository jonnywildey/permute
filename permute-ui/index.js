// const permuteLibrary = require("./permute-library");

// console.log("start");
// permuteLibrary.registerUpdates((state) => {
//   console.log(state);
// });

// permuteLibrary.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");

// permuteLibrary.runProcess();

// setTimeout(() => { }, 20000);


"use strict";
const { databaseNew, databaseClose, runProcess, addFile, setStateCallback } = require("./permute-library");



// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class Processor {
  constructor() {
    this.db = databaseNew();
  }

  close() {
    databaseClose.call(this.db);
  }

  runProcess() {
    return runProcess.call(this.db);
  }

  addFile(file) {
    return addFile.call(this.db, file);
  }

  setStateCallback(cb) {
    return setStateCallback.call(this.db, cb);
  }
}

const runDb = async () => {
  const processor = new Processor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.setStateCallback((state) => {
    console.log(state);
  });
  processor.runProcess();
}

runDb();

setTimeout(() => { }, 30000);

