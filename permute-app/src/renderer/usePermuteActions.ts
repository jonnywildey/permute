import { useCallback, useState } from 'react';
import { useToast } from '@chakra-ui/react';
import { debounce } from 'lodash';

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

export function usePermuteActions() {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const toast = useToast();

  // Memoized state refresh function
  const refreshState = useCallback(
    debounce(async () => {
      try {
        const permuteState = await window.Electron.ipcRenderer.getState();
        setState(prevState => ({ ...prevState, permuteState }));
      } catch (error) {
        console.error('Failed to refresh state:', error);
        toast({
          description: 'Failed to refresh application state',
          status: 'error',
          duration: 5000,
          isClosable: true,
        });
      }
    }, 100),
    [toast]
  );

  // File management actions
  const fileActions = {
    addFiles: useCallback(async (files: string[]) => {
      await Promise.all(files.map(f => window.Electron.ipcRenderer.addFile(f)));
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ permuteState });
    }, []),

    removeFile: useCallback(async (file: string) => {
      await window.Electron.ipcRenderer.removeFile(file);
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ permuteState });
    }, []),

    showFile: useCallback(async (file: string) => {
      await window.Electron.ipcRenderer.showFile(file);
    }, []),

    deleteOutputFile: useCallback(async (file: string) => {
      await window.Electron.ipcRenderer.deleteOutputFile(file);
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ permuteState });
    }, []),

    deleteAllOutputFiles: useCallback(async () => {
      await window.Electron.ipcRenderer.deleteAllOutputFiles();
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ permuteState });
    }, []),
  };

  // Processing actions
  const processingActions = {
    runProcessor: useCallback(async () => {
      const onFinished = (pState: IPermuteState) => {
        if (pState.error) {
          toast({
            description: pState.error,
            status: 'error',
            duration: 5000,
            isClosable: true,
          });
        } else {
          toast({
            description: `${pState.files.length * pState.permutations} files permuted!`,
            status: 'success',
            duration: 5000,
            isClosable: true,
          });
        }
        setState({ permuteState: pState });
      };
      window.Electron.ipcRenderer.runProcessor(refreshState, onFinished);
    }, [refreshState, toast]),

    reverseFile: useCallback(async (file: string) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, processing: true }
      }));
      const onFinished = (pState: IPermuteState) => {
        setState({ permuteState: pState });
      };
      window.Electron.ipcRenderer.reverseFile(refreshState, onFinished, file);
    }, [refreshState]),

    trimFile: useCallback(async (file: string) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, processing: true }
      }));
      const onFinished = (pState: IPermuteState) => {
        setState({ permuteState: pState });
      };
      window.Electron.ipcRenderer.trimFile(refreshState, onFinished, file);
    }, [refreshState]),

    cancelProcessing: useCallback(async () => {
      await window.Electron.ipcRenderer.cancel();
      refreshState();
    }, [refreshState]),
  };

  // Settings actions with optimistic updates
  const settingsActions = {
    setDepth: useCallback(async (depth: number) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, permutationDepth: depth }
      }));
      await window.Electron.ipcRenderer.setDepth(depth);
      refreshState();
    }, [refreshState]),

    setPermutations: useCallback(async (permutations: number) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, permutations }
      }));
      await window.Electron.ipcRenderer.setPermutations(permutations);
      refreshState();
    }, [refreshState]),

    setNormalised: useCallback(async (normaliseAtEnd: boolean) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, normaliseAtEnd }
      }));
      await window.Electron.ipcRenderer.setNormalised(normaliseAtEnd);
      refreshState();
    }, [refreshState]),

    setTrimAll: useCallback(async (trimAll: boolean) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, trimAll }
      }));
      await window.Electron.ipcRenderer.setTrimAll(trimAll);
      refreshState();
    }, [refreshState]),

    setInputTrail: useCallback(async (inputTrail: number) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, inputTrail }
      }));
      await window.Electron.ipcRenderer.setInputTrail(inputTrail);
      refreshState();
    }, [refreshState]),

    setOutputTrail: useCallback(async (outputTrail: number) => {
      setState(prev => ({
        permuteState: { ...prev.permuteState, outputTrail }
      }));
      await window.Electron.ipcRenderer.setOutputTrail(outputTrail);
      refreshState();
    }, [refreshState]),
  };

  // Processor management actions
  const processorActions = {
    setProcessorEnabled: useCallback((name: string, enable: boolean) => {
      if (enable) {
        window.Electron.ipcRenderer.addProcessor(name);
      } else {
        window.Electron.ipcRenderer.removeProcessor(name);
      }
      refreshState();
    }, [refreshState]),

    selectAllProcessors: useCallback(async () => {
      await window.Electron.ipcRenderer.selectAllProcessors();
      refreshState();
    }, [refreshState]),

    deselectAllProcessors: useCallback(async () => {
      await window.Electron.ipcRenderer.deselectAllProcessors();
      refreshState();
    }, [refreshState]),
  };

  // Scene management actions
  const sceneActions = {
    handleSaveScene: useCallback(() => {
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
    }, [refreshState, toast]),

    handleLoadScene: useCallback(() => {
      window.Electron.ipcRenderer.loadScene(async (response: LoadSceneResponse | string) => {
        try {
          if (typeof response === 'string') {
            // Legacy string response handling
            const permuteState = await window.Electron.ipcRenderer.getState();
            setState({ permuteState });
            toast({
              description: 'Scene loaded successfully',
              status: 'success',
              duration: 3000,
              isClosable: true,
            });
          } else {
            // New LoadSceneResponse handling
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
    }, [toast]),

    setOutput: useCallback(async () => {
      window.Electron.ipcRenderer.openOutputDialog(([output]: [string]) => {
        window.Electron.ipcRenderer.setOutput(output);
        refreshState();
      });
    }, [refreshState]),

    setCreateSubdirectories: useCallback(async (createSubfolders: boolean) => {
      await window.Electron.ipcRenderer.setCreateSubdirectories(createSubfolders);
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ permuteState });
    }, []),
  };

  return {
    state,
    setState,
    refreshState,
    ...fileActions,
    ...processingActions,
    ...settingsActions,
    ...processorActions,
    ...sceneActions,
  };
} 