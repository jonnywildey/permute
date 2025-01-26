const { createPermuteProcessor } = require(".");

const run = () => {
  const processor = createPermuteProcessor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/amen.wav");
  processor.reverseFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/amen.wav",
    (state) => {
      console.log("update", state)
    }, (state) => {
      console.log("finished", state);
      process.exit();
    });
};

run();


