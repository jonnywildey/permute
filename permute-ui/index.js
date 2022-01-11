const p = require('.');

p.init();

class Permute {
  constructor() {
    this.permute = p.init();
  }

  getState() {
    p.getState(this.permute);
  }

  run() {
    p.run(this.permute);
  }
}

const permute = new Permute();

const state = permute.getState();

console.log(state);

permute.run();