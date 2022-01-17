const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('Electron', {
  ipcRenderer: {
    openOutputDialog(f) {
      ipcRenderer.once('open-output-dialog', (event, ...args) => f(...args));
      ipcRenderer.send('open-output-dialog');
    },
    runProcessor(updateFn, completeFn) {
      const listener = (event, ...args) => {
        console.log("update");
        updateFn(...args)
      };
      ipcRenderer.on('run-processor-update', listener);
      ipcRenderer.once('run-processor-ended', (event, ...args) => {
        console.log("ended");
        ipcRenderer.removeListener('run-processor-update', listener);
        completeFn(...args)
      });
      ipcRenderer.send('run-processor');

    },
    myPing() {
      ipcRenderer.send('ipc-example', 'ping');
    },
    on(channel, func) {
      const validChannels = ['ipc-example'];
      if (validChannels.includes(channel)) {
        // Deliberately strip event as it includes `sender`
        ipcRenderer.on(channel, (event, ...args) => func(...args));
      }
    },
    once(channel, func) {
      const validChannels = ['ipc-example', 'open-output-dialog'];
      if (validChannels.includes(channel)) {
        // Deliberately strip event as it includes `sender`
        ipcRenderer.once(channel, (event, ...args) => func(...args));
      }
    },
  },
});
