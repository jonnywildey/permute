const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('Electron', {
  ipcRenderer: {
    openOutputDialog(f) {
      ipcRenderer.once('open-output-dialog', (event, ...args) => f(...args));
      ipcRenderer.send('open-output-dialog');
    },
    runProcessor(updateFn, completeFn) {
      const listener = (event, ...args) => {
        updateFn(...args);
      };
      ipcRenderer.on('run-processor-update', listener);
      ipcRenderer.once('run-processor-ended', (event, ...args) => {
        ipcRenderer.removeListener('run-processor-update', listener);
        completeFn(...args);
      });
      ipcRenderer.send('run-processor');
    },
    addFile(file) {
      ipcRenderer.send('add-file', file);
    },
    removeFile(file) {
      ipcRenderer.send('remove-file', file);
    },
    removeProcessor(name) {
      ipcRenderer.send('remove-processor', name);
    },
    addProcessor(name) {
      ipcRenderer.send('add-processor', name);
    },
    setOutput(output) {
      ipcRenderer.send('set-output', output);
    },
    setDepth(depth) {
      ipcRenderer.send('set-depth', depth);
    },
    setNormalised(normalised) {
      ipcRenderer.send('set-normalised', normalised);
    },
    setPermutations(permutations) {
      ipcRenderer.send('set-permutations', permutations);
    },
    setInputTrail(trail) {
      ipcRenderer.send('set-input-trail', trail);
    },
    setOutputTrail(trail) {
      ipcRenderer.send('set-output-trail', trail);
    },
    showFile(file) {
      ipcRenderer.send('show-file', file);
    },
    getState() {
      return new Promise((res) => {
        ipcRenderer.once('get-state', (event, ...args) => res(...args));
        ipcRenderer.send('get-state');
      });
    },
    getFileStats(files) {
      return new Promise((res) => {
        ipcRenderer.once('get-file-stats', (event, ...args) => res(...args));
        ipcRenderer.send('get-file-stats', files);
      });
    },
  },
});
