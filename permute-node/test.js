const { createPermuteProcessor } = require(".");

const run = () => {
  const processor = createPermuteProcessor();
  processor.loadSettings("/Users/jonnywildey/Desktop/invalidscene.json", (state) => {
    console.log("finished", state);
    process.exit();
  }, (state) => {
    console.log("error", state);
    process.exit();
  });

  // processor.setDepth(0);
  // processor.setPermutations(1);
  // processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/cool.wav");
  // processor.addProcessor("Flange");
  // processor.setOutput("/Users/jonnywildey/rustcode/permute/permute-core/renders/")
  // processor.runProcess(
  //   (state) => {
  //     // console.log("update", state)
  //   }, (state) => {
  //     console.log("finished", state);
  //     process.exit();
  //   });
  // setTimeout(() => {
  //   processor.cancel();
  // }, 10);
  //   processor.reverseFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/cool.wav",
  //     (state) => {
  //       // console.log("update", state)
  //     }, (state) => {
  //       console.log("finished", state);
  //       process.exit();
  //     });

  // };


}
run();
