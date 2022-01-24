const { createPermuteProcessor } = require(".");

const run = () => {
  const processor = createPermuteProcessor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.runProcess((state) => {
    console.log("update", state)
    if (state.finished) {
      process.exit();
    }
  });
};

run();


