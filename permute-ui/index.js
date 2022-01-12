const permuteLibrary = require("./permute-library");


permuteLibrary.registerUpdates((counter) => {
  console.log(counter);
  permuteLibrary.increment();
});


