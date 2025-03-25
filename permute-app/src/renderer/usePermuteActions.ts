import { useCallback } from 'react';
import { useToast } from '@chakra-ui/react';
import debounce from 'lodash/debounce';
import type { IPermuteState } from 'permute-node';

export interface IPermutationInput {
  path: string;
  name: string;
  durationSec: number;
  image: string;
}

export interface IPermutationOutput {
  path: string;
  progress: number;
  image: string;
  processors: string[];
  name: string;
  durationSec: number;
  deleted: boolean;
}

export interface IPermuteState {
  output: string;
  error: string;
  processing: boolean;
  highSampleRate: boolean;
  inputTrail: number;
  outputTrail: number;
  files: IPermutationInput[];
  permutations: number;
  permutationDepth: number;
  processorCount: number;
  processorPool: string[];
  allProcessors: string[];
  normaliseAtEnd: boolean;
  trimAll: boolean;
  createSubdirectories: boolean;
  permutationOutputs: IPermutationOutput[];
  viewedWelcome: boolean;
}

export interface IAppState {
  permuteState: IPermuteState;
}

interface LoadSceneResponse {
  success: boolean;
  error?: string;
  filePath?: string;
}

const defaultAppState: IAppState = {
  permuteState: {
    allProcessors: [],
    files: [],
    permutationOutputs: [],
    processorPool: [],
    viewedWelcome: false,
    output: '',
    error: '',
    processing: false,
    highSampleRate: false,
    inputTrail: 0,
    outputTrail: 0,
    permutations: 1,
    permutationDepth: 1,
    processorCount: 0,
    normaliseAtEnd: false,
    trimAll: false,
    createSubdirectories: false,
  },
};

const DEBOUNCE_DELAY = 150; // ms

export const usePermuteActions = (
  state: IAppState,
  setState: (state: IAppState) => void,
) => {
  const toast = useToast();

  // Debounced state refresh to prevent too frequent updates
  const refreshState = useCallback(
    debounce(async () => {
      try {
        const permuteState = await window.Electron.ipcRenderer.getState();
        setState({ permuteState });
      } catch (error) {
        console.error('Failed to refresh state:', error);
      }
    }, DEBOUNCE_DELAY),
    [setState]
  );

  // File management actions
  const fileActions = {
    addFiles: useCallback(async (files: string[]) => {
      for (const file of files) {
        await window.Electron.ipcRenderer.addFile(file);
      }
      refreshState();
    }, [refreshState]),

    removeFile: useCallback(async (file: string) => {
      await window.Electron.ipcRenderer.removeFile(file);
      refreshState();
    }, [refreshState]),

    showFile: useCallback((file: string) => {
      window.Electron.ipcRenderer.showFile(file);
    }, []),

    deleteOutputFile: useCallback(async (file: string) => {
      await window.Electron.ipcRenderer.deleteOutputFile(file);
      refreshState();
    }, [refreshState]),

    deleteAllOutputFiles: useCallback(async () => {
      await window.Electron.ipcRenderer.deleteAllOutputFiles();
      refreshState();
    }, [refreshState]),
  };

  // Processing actions
  const processingActions = {
    runProcessor: useCallback(async () => {
      window.Electron.ipcRenderer.runProcessor(
        (state: IPermuteState) => {
          setState({ permuteState: state });
        },
        (state: IPermuteState) => {
          setState({ permuteState: state });
        }
      );
    }, [setState]),

    reverseFile: useCallback(async (file: string) => {
      window.Electron.ipcRenderer.reverseFile(
        file,
        (state: IPermuteState) => {
          setState({ permuteState: state });
        },
        (state: IPermuteState) => {
          setState({ permuteState: state });
        }
      );
    }, [setState]),

    trimFile: useCallback(async (file: string) => {
      window.Electron.ipcRenderer.trimFile(
        file,
        (state: IPermuteState) => {
          setState({ permuteState: state });
        },
        (state: IPermuteState) => {
          setState({ permuteState: state });
        }
      );
    }, [setState]),

    cancelProcessing: useCallback(() => {
      window.Electron.ipcRenderer.cancel();
      refreshState();
    }, [refreshState]),
  };

  // Settings actions with optimistic updates
  const setDepth = useCallback(
    debounce((depth: number) => {
      setState(prev => ({
        ...prev,
        permuteState: { ...prev.permuteState, permutationDepth: depth }
      }));
      window.Electron.ipcRenderer.setDepth(depth);
      refreshState();
    }, DEBOUNCE_DELAY),
    [setState, refreshState]
  );

  const setPermutations = useCallback(
    debounce((permutations: number) => {
      setState(prev => ({
        ...prev,
        permuteState: { ...prev.permuteState, permutations }
      }));
      window.Electron.ipcRenderer.setPermutations(permutations);
      refreshState();
    }, DEBOUNCE_DELAY),
    [setState, refreshState]
  );

  const setNormalised = useCallback((normalised: boolean) => {
    setState(prev => ({
      ...prev,
      permuteState: { ...prev.permuteState, normaliseAtEnd: normalised }
    }));
    window.Electron.ipcRenderer.setNormalised(normalised);
    refreshState();
  }, [setState, refreshState]);

  const setTrimAll = useCallback((trimAll: boolean) => {
    setState(prev => ({
      ...prev,
      permuteState: { ...prev.permuteState, trimAll }
    }));
    window.Electron.ipcRenderer.setTrimAll(trimAll);
    refreshState();
  }, [setState, refreshState]);

  const setInputTrail = useCallback(
    debounce((trail: number) => {
      setState(prev => ({
        ...prev,
        permuteState: { ...prev.permuteState, inputTrail: trail }
      }));
      window.Electron.ipcRenderer.setInputTrail(trail);
      refreshState();
    }, DEBOUNCE_DELAY),
    [setState, refreshState]
  );

  const setOutputTrail = useCallback(
    debounce((trail: number) => {
      setState(prev => ({
        ...prev,
        permuteState: { ...prev.permuteState, outputTrail: trail }
      }));
      window.Electron.ipcRenderer.setOutputTrail(trail);
      refreshState();
    }, DEBOUNCE_DELAY),
    [setState, refreshState]
  );

  const setProcessorEnabled = useCallback(async (name: string, enabled: boolean) => {
    if (enabled) {
      await window.Electron.ipcRenderer.addProcessor(name);
    } else {
      await window.Electron.ipcRenderer.removeProcessor(name);
    }
    refreshState();
  }, [refreshState]);

  const selectAllProcessors = useCallback(async () => {
    await window.Electron.ipcRenderer.selectAllProcessors();
    refreshState();
  }, [refreshState]);

  const deselectAllProcessors = useCallback(async () => {
    await window.Electron.ipcRenderer.deselectAllProcessors();
    refreshState();
  }, [refreshState]);

  const setOutput = useCallback(async (output: string) => {
    await window.Electron.ipcRenderer.setOutput(output);
    refreshState();
  }, [refreshState]);

  const setCreateSubdirectories = useCallback(async (create: boolean) => {
    await window.Electron.ipcRenderer.setCreateSubdirectories(create);
    refreshState();
  }, [refreshState]);

  const handleSaveScene = useCallback(async () => {
    window.Electron.ipcRenderer.saveScene((filePath: string) => {
      if (filePath) {
        toast({
          description: 'Scene saved successfully',
          status: 'success',
          duration: 3000,
          isClosable: true,
        });
        refreshState();
      }
    });
  }, [refreshState, toast]);

  const handleLoadScene = useCallback(async () => {
    window.Electron.ipcRenderer.loadScene(async (response: any) => {
      try {
        if (typeof response === 'string') {
          const permuteState = await window.Electron.ipcRenderer.getState();
          setState({ permuteState });
          toast({
            description: 'Scene loaded successfully',
            status: 'success',
            duration: 3000,
            isClosable: true,
          });
        } else {
          if (response.success) {
            const permuteState = await window.Electron.ipcRenderer.getState();
            setState({ permuteState });
            toast({
              description: 'Scene loaded successfully',
              status: 'success',
              duration: 3000,
              isClosable: true,
            });
          } else {
            toast({
              description: response.error || 'Error loading scene',
              status: 'error',
              duration: 5000,
              isClosable: true,
            });
          }
        }
      } catch (error) {
        console.error('Error loading scene:', error);
        toast({
          description: 'Failed to load scene',
          status: 'error',
          duration: 5000,
          isClosable: true,
        });
      }
    });
  }, [setState, toast]);

  return {
    state,
    setState,
    refreshState,
    ...fileActions,
    ...processingActions,
    setDepth,
    setPermutations,
    setNormalised,
    setTrimAll,
    setInputTrail,
    setOutputTrail,
    setProcessorEnabled,
    selectAllProcessors,
    deselectAllProcessors,
    setOutput,
    setCreateSubdirectories,
    handleSaveScene,
    handleLoadScene,
  };
}; 