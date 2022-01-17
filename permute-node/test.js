const { createPermuteProcessor } = require(".");

const run = async () => {
  const processor = createPermuteProcessor();
  processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");
  processor.runProcess((state) => {
    console.log("update")
  }, () => {
    console.log("ended")
  });
}

run();


