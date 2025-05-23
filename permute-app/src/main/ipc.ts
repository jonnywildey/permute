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

ipcMain.on('save-scene-dialog', async (event) => {
  const result = await dialog.showSaveDialog({
    filters: [{ name: 'Scene Files', extensions: ['json'] }],
    defaultPath: 'scene.json'
  });
  if (!result.canceled && result.filePath) {
    processor.saveSettings(result.filePath);
    event.reply('save-scene-dialog', result.filePath);
  }
});

ipcMain.on('load-scene-dialog', async (event) => {
  const result = await dialog.showOpenDialog({
    filters: [{ name: 'Scene Files', extensions: ['json'] }],
    properties: ['openFile']
  });
  if (!result.canceled && result.filePaths.length > 0) {
    try {
      processor.loadSettings(result.filePaths[0]);
      // Get state to check for errors
      const state = await processor.getState();
      if (state.error) {
        event.reply('load-scene-dialog', {
          success: false,
          error: state.error
        });
      } else {
        event.reply('load-scene-dialog', {
          success: true,
          filePath: result.filePaths[0]
        });
      }
    } catch (error: any) {
      console.error(error);
      event.reply('load-scene-dialog', {
        success: false,
        error: `failed to load scene`
      });
    }
  }
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
  processor.setOutput(String(param));
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

ipcMain.on('delete-all-output-files', async () => {
  processor.deleteAllOutputFiles();
});

ipcMain.on('set-create-subdirectories', async (_, param) => {
  processor.setCreateSubdirectories(param);
});

ipcMain.on('select-all-processors', async () => {
  processor.selectAllProcessors();
});

ipcMain.on('deselect-all-processors', async () => {
  processor.deselectAllProcessors();
});

ipcMain.on('set-viewed-welcome', async (_, param) => {
  processor.setViewedWelcome(param);
});

ipcMain.on('set-max-stretch', async (_, param) => {
  processor.setMaxStretch(param);
});

app.on('before-quit', () => {
  processor.saveSettings(configPath);
});
