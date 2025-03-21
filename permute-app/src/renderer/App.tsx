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
import { useEffect, useState } from 'react';
import { Files } from './Files';
import { TopBar } from './TopBar';
import { Output } from './Output';
import { BottomBar } from './BottomBar';
import { theme } from './theme';
import { Processors } from './Processors';
import { Welcome } from './Welcome';
import { CreateAudioContext } from './AudioContext';

export interface IAppState {
  permuteState: IPermuteState;
}

const defaultAppState: IAppState = {
  permuteState: {
    allProcessors: [],
    files: [],
    permutationOutputs: [],
    processorPool: [],
  } as any,
};

const Content = ({ onOpen }: { onOpen: () => void }) => {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const toast = useToast();
  const { colorMode } = useColorMode();

  useEffect(() => {
    document.body.setAttribute('data-theme', colorMode);
  }, [colorMode]);

  const refreshState = async () => {
    try {
      const permuteState = await window.Electron.ipcRenderer.getState();
      setState({ ...state, permuteState });
    } catch (error) {
      console.error('Failed to refresh state:', error);
      toast({
        description: 'Failed to refresh application state',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  };

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

  const cancelProcessing = async () => {
    window.Electron.ipcRenderer.cancel();
    refreshState();
  };

  const setCreateSubdirectories = async (createSubfolders: boolean) => {
    window.Electron.ipcRenderer.setCreateSubdirectories(createSubfolders);
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ permuteState });
  };


  return (
    <Grid
      templateRows="repeat(24, 1fr)"
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
      />
      <Output
        output={output}
        setOutput={setOutput}
        showFile={showFile}
        permutationOutputs={permutationOutputs}
        reverseFile={reverseFile}
        trimFile={trimFile}
        deleteOutputFile={deleteOutputFile}
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
  const { isOpen, onClose, onOpen } = useDisclosure({
    defaultIsOpen: true,
  });

  useEffect(() => {
    // Preload the background image
    const img = new Image();
    img.src = require('../img/bg2.png');

    // Wait for both the timeout and image load
    Promise.all([
      new Promise(resolve => setTimeout(resolve, 1500)),
      new Promise(resolve => {
        if (img.complete) {
          resolve(null);
        } else {
          img.onload = () => resolve(null);
        }
      })
    ]).then(() => {
      setLoading(false);
      // Add a small delay before showing the main content
      setTimeout(() => {
        setShowContent(true);
      }, 500);
    });
  }, []);

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
            <Welcome isOpen={isOpen} onClose={onClose} />
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
