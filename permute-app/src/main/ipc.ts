import { ipcMain, dialog, app, shell } from 'electron';
import { createPermuteProcessor, IPermuteState } from 'permute-node';
import path from 'path';

const configPath = path.join(app.getPath('userData'), 'config.json');
export const processor = createPermuteProcessor();

// Load settings immediately when the processor is created
processor.loadSettings(configPath);

ipcMain.on('open-output-dialog', async (event) => {
  const result = await dialog.showOpenDialog({ properties: ['openDirectory'] });
  console.log(result.filePaths);
  event.reply('open-output-dialog', result.filePaths);
});

ipcMain.on('run-processor', async (event) => {
  processor.runProcess(
    (state: IPermuteState) => {
      event.reply('run-processor-update', state);
    },
    (state: IPermuteState) => {
      event.reply('run-processor-ended', state);
    }
  );
});

ipcMain.on('cancel', async () => {
  processor.cancel();
});

ipcMain.on('reverse-file', async (event, file) => {
  processor.reverseFile(
    file,
    (state: IPermuteState) => {
      event.reply('reverse-file-update', state);
    },
    (state: IPermuteState) => {
      event.reply('reverse-file-ended', state);
    }
  );
});
ipcMain.on('trim-file', async (event, file) => {
  processor.trimFile(
    file,
    (state: IPermuteState) => {
      event.reply('trim-file-update', state);
    },
    (state: IPermuteState) => {
      event.reply('trim-file-ended', state);
    }
  );
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
ipcMain.on('set-trim-all', async (_, param) => {
  processor.setTrimAll(param);
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

ipcMain.on('delete-output-file', async (_, file) => {
  processor.deleteOutputFile(file);
});

ipcMain.on('set-create-subdirectories', async (_, param) => {
  processor.setCreateSubdirectories(param);
});

app.on('before-quit', () => {
  processor.saveSettings(configPath);
});
