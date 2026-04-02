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
import type { IPermuteState } from './types';
import { useEffect, useState, useMemo, memo, useCallback } from 'react';
import { Files } from './Files';
import { TopBar } from './TopBar';
import { Output } from './Output';
import { BottomBar } from './BottomBar';
import { theme } from './theme';
import { Processors } from './Processors';
import { Welcome } from './Welcome';
import { CreateAudioContext } from './AudioContext';
import { debounce } from 'lodash';
import * as bridge from './bridge';
import bg2Url from './img/bg2.png';

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

const MemoizedFiles = memo(Files);
const MemoizedTopBar = memo(TopBar);
const MemoizedOutput = memo(Output);
const MemoizedBottomBar = memo(BottomBar);
const MemoizedProcessors = memo(Processors);

const Content = () => {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const toast = useToast();
  const { colorMode } = useColorMode();

  useEffect(() => {
    document.body.setAttribute('data-theme', colorMode);
  }, [colorMode]);

  const refreshState = useCallback(
    debounce(async () => {
      try {
        const permuteState = await bridge.getState();
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

  useEffect(() => {
    return () => {
      refreshState.cancel();
    };
  }, [refreshState]);

  useEffect(() => {
    const setup = async () => {
      try {
        const permuteState = await bridge.getState();
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
  }, [toast, setState]);

  const { isOpen, onClose, onOpen } = useDisclosure({ defaultIsOpen: false });

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
    processing,
    processorPool,
    permutationOutputs,
    createSubdirectories,
    maxStretch,
  } = useMemo(() => state.permuteState, [state.permuteState]);

  const gridConfig = useMemo(() => ({
    templateRows: "repeat(26, 1fr)",
    templateColumns: "repeat(12, 1fr)",
    gap: 3,
    padding: 2,
    width: "100%",
    height: "100vh"
  }), []);

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
        const description = `${pState.files.length * pState.permutations} files permuted!`;
        toast({
          description,
          status: 'success',
          duration: 5000,
          isClosable: true,
        });
      }
      setState({ ...state, permuteState: pState });
    };
    setState({ permuteState: { ...state.permuteState, processing: true } });
    bridge.runProcessor(refreshState, onFinished);
  };

  const reverseFile = async (file: string) => {
    setState({ permuteState: { ...state.permuteState, processing: true } });
    const onFinished = (pState: IPermuteState) => {
      setState({ ...state, permuteState: pState });
    };
    bridge.reverseFile(refreshState, onFinished, file);
  };

  const trimFile = async (file: string) => {
    setState({ permuteState: { ...state.permuteState, processing: true } });
    const onFinished = (pState: IPermuteState) => {
      setState({ ...state, permuteState: pState });
    };
    bridge.trimFile(refreshState, onFinished, file);
  };

  const setDepth = async (depth: number) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, permutationDepth: depth }
    }));
    await bridge.setDepth(depth);
    refreshState();
  };

  const setPermutations = async (permutations: number) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, permutations }
    }));
    await bridge.setPermutations(permutations);
    refreshState();
  };

  const setNormalised = async (normaliseAtEnd: boolean) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, normaliseAtEnd }
    }));
    await bridge.setNormalised(normaliseAtEnd);
    refreshState();
  };

  const setTrimAll = async (trimAll: boolean) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, trimAll }
    }));
    await bridge.setTrimAll(trimAll);
    refreshState();
  };

  const setInputTrail = async (inputTrail: number) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, inputTrail }
    }));
    await bridge.setInputTrail(inputTrail);
    refreshState();
  };

  const setOutputTrail = async (outputTrail: number) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, outputTrail }
    }));
    await bridge.setOutputTrail(outputTrail);
    refreshState();
  };

  const addFiles = async (files: string[]) => {
    await Promise.all(files.map((f) => bridge.addFile(f)));
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const removeFile = async (file: string) => {
    await bridge.removeFile(file);
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const clearAllFiles = async () => {
    await bridge.clearAllFiles();
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const showFile = async (file: string) => {
    bridge.showFile(file);
  };

  const deleteOutputFile = async (file: string) => {
    await bridge.deleteOutputFile(file);
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const deleteAllOutputFiles = async () => {
    await bridge.deleteAllOutputFiles();
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const setOutput = async () => {
    const chosen = await bridge.openOutputDialog();
    if (chosen) {
      refreshState();
    }
  };

  const setProcessorEnabled = (name: string, enable: boolean) => {
    if (enable) {
      bridge.addProcessor(name);
    } else {
      bridge.removeProcessor(name);
    }
    refreshState();
  };

  const selectAllProcessors = async () => {
    await bridge.selectAllProcessors();
    refreshState();
  };

  const deselectAllProcessors = async () => {
    await bridge.deselectAllProcessors();
    refreshState();
  };

  const cancelProcessing = async () => {
    bridge.cancel();
    refreshState();
  };

  const setCreateSubdirectories = async (createSubfolders: boolean) => {
    await bridge.setCreateSubdirectories(createSubfolders);
    const permuteState = await bridge.getState();
    setState({ permuteState });
  };

  const handleSaveScene = async () => {
    const filePath = await bridge.saveScene();
    if (filePath) {
      toast({
        description: 'Scene saved successfully',
        status: 'success',
        duration: 3000,
        isClosable: true,
      });
      refreshState();
    }
  };

  const handleLoadScene = async () => {
    const response = await bridge.loadScene();
    if (response.success) {
      const permuteState = await bridge.getState();
      setState({ permuteState });
      toast({
        description: 'Scene loaded successfully',
        status: 'success',
        duration: 3000,
        isClosable: true,
      });
    } else {
      toast({
        description: 'Error loading scene',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  };

  const setMaxStretch = async (maxStretch: number) => {
    setState(prevState => ({
      permuteState: { ...prevState.permuteState, maxStretch }
    }));
    await bridge.setMaxStretch(maxStretch);
    refreshState();
  };

  return (
    <Grid {...gridConfig}>
      <Welcome isOpen={isOpen} onClose={onClose} />
      <MemoizedTopBar
        openWelcome={onOpen}
        createSubdirectories={createSubdirectories}
        onCreateSubdirectoriesChange={setCreateSubdirectories}
        onSaveScene={handleSaveScene}
        onLoadScene={handleLoadScene}
        maxStretch={maxStretch}
        onMaxStretchChange={setMaxStretch}
      />
      <MemoizedFiles
        files={files}
        addFiles={addFiles}
        removeFile={removeFile}
        clearAllFiles={clearAllFiles}
        showFile={showFile}
      />
      <MemoizedProcessors
        allProcessors={allProcessors}
        processorPool={processorPool}
        setProcessorEnabled={setProcessorEnabled}
        onSelectAll={selectAllProcessors}
        onDeselectAll={deselectAllProcessors}
      />
      <MemoizedOutput
        output={output}
        setOutput={setOutput}
        showFile={showFile}
        permutationOutputs={permutationOutputs}
        reverseFile={reverseFile}
        trimFile={trimFile}
        deleteOutputFile={deleteOutputFile}
        deleteAllOutputFiles={deleteAllOutputFiles}
      />
      <MemoizedBottomBar
        permutationOutputs={permutationOutputs}
        runProcessor={runProcessor}
        processing={processing}
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
  const { isOpen, onClose } = useDisclosure({ defaultIsOpen: true });

  const handleWelcomeClose = () => {
    bridge.setViewedWelcome(true);
    onClose();
  };

  useEffect(() => {
    Promise.all([
      bridge.getState().then(permuteState => {
        setState({ permuteState });
        return permuteState;
      }),
      new Promise(resolve => {
        const img = new Image();
        img.src = bg2Url;
        if (img.complete) {
          resolve(null);
        } else {
          img.onload = () => resolve(null);
        }
      }),
      new Promise(resolve => setTimeout(resolve, 1500)),
    ]).then(([permuteState]) => {
      setLoading(false);
      setTimeout(() => {
        setShowContent(true);
      }, 500);
      if ((permuteState as IPermuteState).viewedWelcome) {
        onClose();
      }
    });
  }, [onClose]);

  const shouldShowWelcome = !loading && isOpen && !state.permuteState.viewedWelcome;

  return (
    <ChakraProvider theme={theme}>
      <CreateAudioContext>
        {loading ? (
          <>
            <div className="font_preload" style={{ opacity: 0 }}>
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
              {showContent && <Content />}
            </Box>
          </>
        )}
      </CreateAudioContext>
    </ChakraProvider>
  );
}
