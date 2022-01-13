const permuteLibrary = require("./permute-library");


permuteLibrary.registerUpdates((state) => {
  console.log(state);
});

permuteLibrary.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");

permuteLibrary.runProcess();

setTimeout(() => { }, 20000);


