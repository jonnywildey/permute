import { ipcMain, dialog, app } from 'electron';
import { createPermuteProcessor, IPermuteState } from "permute-node";
import { shell } from 'electron';
import path from "path";

const configPath = path.join(app.getPath('userData'), "config.json");
export const processor = createPermuteProcessor();

ipcMain.on('open-output-dialog', async (event) => {
  const result = await dialog.showOpenDialog({properties: ['openDirectory']});
  console.log(result.filePaths);
  event.reply('open-output-dialog', result.filePaths);
});

ipcMain.on('run-processor', async (event) => {
  processor.runProcess((state: IPermuteState) => {
    event.reply('run-processor-update', state);
  }, (state: IPermuteState) => {
    event.reply('run-processor-ended', state);
  });
});
ipcMain.on('add-file', async (_, file) => {
  processor.addFile(file);
});
ipcMain.on('remove-file', async (_, file) => {
  processor.removeFile(file);
});
ipcMain.on('add-processor', async (_, name) => {
  processor.addProcessor(name);
});
ipcMain.on('remove-processor', async (_, name) => {
  processor.removeProcessor(name);
});
ipcMain.on('set-depth', async (_, param) => {
  processor.setDepth(param);
});
ipcMain.on('set-permutations', async (_, param) => {
  processor.setPermutations(param);
});
ipcMain.on('set-normalised', async (_, param) => {
  processor.setNormalised(param);
});
ipcMain.on('set-input-trail', async (_, param) => {
  processor.setInputTrail(param);
});
ipcMain.on('set-output-trail', async (_, param) => {
  processor.setOutputTrail(param);
});
ipcMain.on('set-output', async (_, param) => {
  processor.setOutput(param);
});
ipcMain.on('get-state', async (event) => {
  const state = await processor.getState();
  event.reply('get-state', state);
});
ipcMain.on('show-file', async (_, file) => {
  shell.showItemInFolder(file);
});

app.on('before-quit', () => {
  processor.saveSettings(configPath);
})
app.on('ready', () => {
  processor.loadSettings(configPath);
})