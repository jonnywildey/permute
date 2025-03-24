import {
  ChakraProvider,
  Grid,
  useDisclosure,
  useToast,
  Spinner,
  Center,
  Text,
  Heading,
  Box,
  useColorMode
} from '@chakra-ui/react';
import type { IPermuteState } from 'permute-node';
import { useEffect, useState, useCallback } from 'react';
import { Files } from './Files';
import { TopBar } from './TopBar';
import { Output } from './Output';
import { BottomBar } from './BottomBar';
import { theme } from './theme';
import { Processors } from './Processors';
import { Welcome } from './Welcome';
import { CreateAudioContext } from './AudioContext';
import { debounce } from 'lodash';

export interface IAppState {
  permuteState: IPermuteState;
}

const defaultAppState: IAppState = {
  permuteState: {
    allProcessors: [],
    files: [],
    permutationOutputs: [],
    processorPool: [],
    viewedWelcome: false,
  } as any,
};

const Content = ({ onOpen }: { onOpen: () => void }) => {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const toast = useToast();
  const { colorMode } = useColorMode();

  useEffect(() => {
    document.body.setAttribute('data-theme', colorMode);
  }, [colorMode]);

  // Create a debounced version of the state refresh
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

  // Cleanup the debounced function when component unmounts
  useEffect(() => {
    return () => {
      refreshState.cancel();
    };
  }, [refreshState]);

  useEffect(() => {
    const setup = async () => {
      try {
        const permuteState = await window.Electron.ipcRenderer.getState();
        if (!permuteState) {
          throw new Error('Failed to load permute state');
        }
        setState({ permuteState });
      } catch (error) {
        console.error('Failed to setup initial state:', error);
        toast({
          description: 'Failed to load saved settings',
          status: 'error',
          duration: 5000,
          isClosable: true,
        });
      }
    };
    setup();
  }, [toast]);

  const {
    allProcessors,
    permutationDepth,
    files,
    output,
    permutations,
    normaliseAtEnd,
    trimAll,
    inputTrail,
    outputTrail,
    processorPool,
    permutationOutputs,
    createSubdirectories,
  } = state.permuteState;

  const runProcessor = async () => {
    const onFinished = (pState: IPermuteState) => {
      if (pState.error) {
        toast({
          description: pState.error,
          status: 'error',
          duration: 5000,
          isClosable: true,
        });
      } else {
        const description = `${pState.files.length * pState.permutations
          } files permuted!`;
        toast({
          description,
          status: 'success',
          duration: 5000,
          isClosable: true,
        });
      }
      setState({ ...state, permuteState: pState });
    };
    window.Electron.ipcRenderer.runProcessor(refreshState, onFinished);
  };
  const reverseFile = async (file: string) => {
    setState({ permuteState: { ...state.permuteState, processing: true } });
    const onFinished = (pState: IPermuteState) => {
      setState({ ...state, permuteState: pState });
    };
    window.Electron.ipcRenderer.reverseFile(refreshState, onFinished, file);
  };
  const trimFile = async (file: string) => {
    setState({ permuteState: { ...state.permuteState, processing: true } });
    const onFinished = (pState: IPermuteState) => {
      setState({ ...state, permuteState: pState });
    };
    window.Electron.ipcRenderer.trimFile(refreshState, onFinished, file);
  };
  const setDepth = async (depth: number) => {
    window.Electron.ipcRenderer.setDepth(depth);
    refreshState();
  };
  const setPermutations = async (permutations: number) => {
    window.Electron.ipcRenderer.setPermutations(permutations);
    refreshState();
  };
  const setNormalised = async (normaliseAtEnd: boolean) => {
    window.Electron.ipcRenderer.setNormalised(normaliseAtEnd);
    refreshState();
  };
  const setTrimAll = async (trimAll: boolean) => {
    window.Electron.ipcRenderer.setTrimAll(trimAll);
    refreshState();
  };
  const setInputTrail = async (inputTrail: number) => {
    window.Electron.ipcRenderer.setInputTrail(inputTrail);
    refreshState();
  };
  const setOutputTrail = async (outputTrail: number) => {
    window.Electron.ipcRenderer.setOutputTrail(outputTrail);
    refreshState();
  };
  const addFiles = async (files: string[]) => {
    files.map((f) => window.Electron.ipcRenderer.addFile(f));
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };
  const removeFile = async (file: string) => {
    window.Electron.ipcRenderer.removeFile(file);
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };
  const showFile = async (file: string) => {
    window.Electron.ipcRenderer.showFile(file);
  };
  const deleteOutputFile = async (file: string) => {
    window.Electron.ipcRenderer.deleteOutputFile(file);
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };
  const deleteAllOutputFiles = async () => {
    window.Electron.ipcRenderer.deleteAllOutputFiles();
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };
  const setOutput = async () => {
    window.Electron.ipcRenderer.openOutputDialog(([output]: [string]) => {
      window.Electron.ipcRenderer.setOutput(output);
      refreshState();
    });
  };

  const setProcessorEnabled = (name: string, enable: boolean) => {
    if (enable) {
      window.Electron.ipcRenderer.addProcessor(name);
    } else {
      window.Electron.ipcRenderer.removeProcessor(name);
    }
    refreshState();
  };

  const selectAllProcessors = async () => {
    await window.Electron.ipcRenderer.selectAllProcessors();
    refreshState();
  };

  const deselectAllProcessors = async () => {
    await window.Electron.ipcRenderer.deselectAllProcessors();
    refreshState();
  };

  const cancelProcessing = async () => {
    window.Electron.ipcRenderer.cancel();
    refreshState();
  };

  const setCreateSubdirectories = async (createSubfolders: boolean) => {
    window.Electron.ipcRenderer.setCreateSubdirectories(createSubfolders);
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };

  const handleSaveScene = () => {
    window.Electron.ipcRenderer.saveScene((filePath) => {
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
  };

  const handleLoadScene = () => {
    window.Electron.ipcRenderer.loadScene(async (filePath) => {
      if (filePath) {
        const permuteState = await window.Electron.ipcRenderer.getState();
        setState({ permuteState });
        toast({
          description: 'Scene loaded successfully',
          status: 'success',
          duration: 3000,
          isClosable: true,
        });
      }
    });
  };

  return (
    <Grid
      templateRows="repeat(26, 1fr)"
      templateColumns="repeat(12, 1fr)"
      gap={3}
      padding={2}
      width="100%"
      height="100vh"
    >
      <TopBar
        openWelcome={onOpen}
        createSubdirectories={createSubdirectories}
        onCreateSubdirectoriesChange={setCreateSubdirectories}
        onSaveScene={handleSaveScene}
        onLoadScene={handleLoadScene}
      />
      <Files
        files={files}
        addFiles={addFiles}
        removeFile={removeFile}
        showFile={showFile}
      />
      <Processors
        allProcessors={allProcessors}
        processorPool={processorPool}
        setProcessorEnabled={setProcessorEnabled}
        onSelectAll={selectAllProcessors}
        onDeselectAll={deselectAllProcessors}
      />
      <Output
        output={output}
        setOutput={setOutput}
        showFile={showFile}
        permutationOutputs={permutationOutputs}
        reverseFile={reverseFile}
        trimFile={trimFile}
        deleteOutputFile={deleteOutputFile}
        deleteAllOutputFiles={deleteAllOutputFiles}
      />
      <BottomBar
        permutationOutputs={permutationOutputs}
        runProcessor={runProcessor}
        processing={state.permuteState.processing}
        depth={permutationDepth}
        permutations={permutations}
        normaliseAtEnd={normaliseAtEnd}
        trimAll={trimAll}
        inputTrail={inputTrail}
        outputTrail={outputTrail}
        setDepth={setDepth}
        setPermutations={setPermutations}
        setNormalised={setNormalised}
        setTrimAll={setTrimAll}
        setInputTrail={setInputTrail}
        setOutputTrail={setOutputTrail}
        processorPool={processorPool}
        files={files}
        output={output}
        cancelProcessing={cancelProcessing}
      />
    </Grid>
  );
};

export default function App() {
  const [loading, setLoading] = useState(true);
  const [showContent, setShowContent] = useState(false);
  const [state, setState] = useState<IAppState>(defaultAppState);
  const { isOpen, onClose, onOpen } = useDisclosure({
    defaultIsOpen: true, // We'll control this after state loads
  });

  const handleWelcomeClose = () => {
    window.Electron.ipcRenderer.setViewedWelcome(true);
    onClose();
  };

  useEffect(() => {
    // Load initial state and preload images in parallel
    Promise.all([
      window.Electron.ipcRenderer.getState().then(permuteState => {
        setState({ permuteState });
        return permuteState;
      }),
      new Promise(resolve => {
        const img = new Image();
        img.src = require('../img/bg2.png');
        if (img.complete) {
          resolve(null);
        } else {
          img.onload = () => resolve(null);
        }
      }),
      // Add minimum loading time for smoother UX
      new Promise(resolve => setTimeout(resolve, 1500))
    ]).then(([permuteState]) => {
      setLoading(false);
      // Add a small delay before showing the main content
      setTimeout(() => {
        setShowContent(true);
      }, 500);

      // Set initial welcome screen state based on loaded state
      if (permuteState.viewedWelcome) {
        onClose();
      }
    });
  }, [onClose]);

  // Don't render welcome screen until we have state
  debugger;
  const shouldShowWelcome = !loading && isOpen && !state.permuteState.viewedWelcome;

  return (
    <ChakraProvider theme={theme}>
      <CreateAudioContext>
        {loading ? (
          <>
            <div className="font_preload" style={{ "opacity": 0 }}>
              <Text>ABC</Text>
              <Heading>ABCDEFG</Heading>
            </div>
            <Center width="100vw" height="100vh">
              <Spinner ml={2} size="xl" color="brand.600" />
            </Center>
            <Center
              width="100vw"
              height="100vh"
              bg="brand.25"
              transition="opacity 0.3s ease-out"
            >
              <Spinner
                size="xl"
                color="brand.600"
                thickness="4px"
                speed="0.8s"
              />
            </Center>
          </>
        ) : (
          <>
            {shouldShowWelcome && <Welcome isOpen={true} onClose={handleWelcomeClose} />}
            <Box
              opacity={showContent ? 1 : 0}
              transform={showContent ? "translateY(0)" : "translateY(20px)"}
              transition="all 0.5s ease-out"
              width="100%"
              height="100%"
            >
              {showContent && <Content onOpen={onOpen} />}
            </Box>
          </>
        )}
      </CreateAudioContext>
    </ChakraProvider>
  );
}
