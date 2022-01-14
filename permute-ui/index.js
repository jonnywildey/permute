// const permuteLibrary = require("./permute-library");

// console.log("start");
// permuteLibrary.registerUpdates((state) => {
//   console.log(state);
// });

// permuteLibrary.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");

// permuteLibrary.runProcess();

// setTimeout(() => { }, 20000);


"use strict";

const { promisify } = require("util");

const { databaseNew, databaseClose, databaseInsert, databaseGetById, runProcess, addFile } = require("./permute-library");

// Convert the DB methods from using callbacks to returning promises
const databaseInsertAsync = promisify(databaseInsert);
const databaseGetByIdAsync = promisify(databaseGetById);

// Wrapper class for the boxed `Database` for idiomatic JavaScript usage
class Processor {
  constructor() {
    this.db = databaseNew();
  }

  // Wrap each method with a delegate to `this.db`
  // This could be node in several other ways, for example binding assignment
  // in the constructor
  insert(name) {
    return databaseInsertAsync.call(this.db, name);
  }

  byId(id) {
    return databaseGetByIdAsync.call(this.db, id);
  }

  close() {
    databaseClose.call(this.db);
  }

  runProcess() {
    return runProcess.call(this.db, (...args) => { console.log(args) });
  }

  addFile(file) {
    return addFile.call(this.db, file);
  }
}

const runDb = async () => {
  const processor = new Processor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.runProcess();
}

runDb();

