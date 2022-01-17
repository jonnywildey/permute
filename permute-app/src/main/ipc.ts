import { ipcMain, dialog } from 'electron';

import { createPermuteProcessor } from "permute-node";
export const processor = createPermuteProcessor();
processor.addFile("/Users/jonnywildey/rustcode/permute/permute-core/examples/vibebeep24.wav");

ipcMain.on('ipc-example', async (event, arg) => {
  const msgTemplate = (pingPong: string) => `IPC test: ${pingPong}`;
  console.log(msgTemplate(arg));
  event.reply('ipc-example', msgTemplate('pong'));
});

ipcMain.on('open-output-dialog', async (event) => {
  const result = await dialog.showOpenDialog({properties: ['openDirectory']});
  console.log(result.filePaths);
  event.reply('open-output-dialog', result.filePaths);
});

ipcMain.on('run-processor', async (event) => {
  processor.runProcess((state) => {
    event.reply('run-processor-update', state);
  }, (state) => {
    event.reply('run-processor-ended', state);
  });
});