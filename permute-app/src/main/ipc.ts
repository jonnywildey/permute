import { ipcMain, dialog } from 'electron';

import { createPermuteProcessor } from "permute-node";
export const processor = createPermuteProcessor();


ipcMain.on('open-output-dialog', async (event) => {
  const result = await dialog.showOpenDialog({properties: ['openDirectory']});
  console.log(result.filePaths);
  event.reply('open-output-dialog', result.filePaths);
});

ipcMain.on('run-processor', async (event) => {
  processor.runProcess((state) => {
    event.reply('run-processor-update', state);
  });
});

ipcMain.on('add-file', async (_, file) => {
  processor.addFile(file);
});

ipcMain.on('set-output', async (_, file) => {
  processor.setOutput(file);
});
ipcMain.on('get-state', async (event) => {
  const state = await processor.getState();
  event.reply('get-state', state);
});