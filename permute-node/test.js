const { createPermuteProcessor } = require(".");

const run = () => {
  const processor = createPermuteProcessor();
  // processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.reverseFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav",
    (state) => {
      console.log("update", state)
    }, (state) => {
      console.log("finished", state);
      process.exit();
    });
};

run();


