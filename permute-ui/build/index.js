"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
const { init, cancel, runProcess, addFile, getStateCallback } = require("../permute-library");
const PERMUTE_POLL_LATENCY = 50;
// Wrapper class for the boxed `Processor` for idiomatic JavaScript usage
class PermuteProcessor {
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
        }, PERMUTE_POLL_LATENCY);
    }
    addFile(file) {
        return addFile.call(this.permuteLibrary, file);
    }
    getStateCallback(cb) {
        return getStateCallback.call(this.permuteLibrary, cb);
    }
}
const run = () => __awaiter(void 0, void 0, void 0, function* () {
    const processor = new PermuteProcessor();
    processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
    processor.getStateCallback((state) => {
        console.log(state);
    });
    processor.runProcess((state) => {
        console.log("woo", state);
    });
});
run();
//# sourceMappingURL=index.js.map