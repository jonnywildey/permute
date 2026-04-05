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
import { useEffect, useState, useMemo, memo, useCallback, useRef } from 'react';
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
import type { PermuteProgressEvent } from './bridge';
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
    return () => { refreshState.cancel(); };
  }, [refreshState]);

  useEffect(() => {
    const setup = async () => {
      try {
        const permuteState = await bridge.getState();
        if (!permuteState) throw new Error('Failed to load permute state');
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

  const { isOpen, onClose, onOpen } = useDisclosure({ defaultIsOpen: false });

  // Direct destructure — no useMemo needed, the object reference is already stable.
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
  } = state.permuteState;

  const gridConfig = useMemo(() => ({
    templateRows: "repeat(26, 1fr)",
    templateColumns: "repeat(12, 1fr)",
    gap: 3,
    padding: 2,
    width: "100%",
    height: "100vh"
  }), []);

  // ─── Processing ─────────────────────────────────────────────────────────────

  const runProcessor = useCallback(() => {
    const expectedCount = files.length * (permutations ?? 0);
    // Clear outputs immediately — backend will stream them back as OutputAdded events.
    setState(prev => ({
      permuteState: { ...prev.permuteState, processing: true, permutationOutputs: [] },
    }));
    bridge.runProcessor(
      (event: PermuteProgressEvent) => {
        setState(prev => {
          const outputs = prev.permuteState.permutationOutputs;
          switch (event.type) {
            case 'outputAdded':
              return {
                permuteState: {
                  ...prev.permuteState,
                  permutationOutputs: [
                    ...outputs,
                    { path: event.path, name: '', progress: 0, processors: event.processors, image: '', durationSec: 0, deleted: false },
                  ],
                },
              };
            case 'outputProgress': {
              const idx = outputs.findIndex(o => o.path === event.path);
              if (idx < 0) return prev;
              const updated = [...outputs];
              updated[idx] = { ...updated[idx], progress: event.progress };
              return { permuteState: { ...prev.permuteState, permutationOutputs: updated } };
            }
            case 'outputCompleted': {
              const idx = outputs.findIndex(o => o.path === event.path);
              if (idx < 0) return prev;
              const updated = [...outputs];
              updated[idx] = { ...updated[idx], name: event.name, image: event.image, durationSec: event.durationSec, progress: 100 };
              return { permuteState: { ...prev.permuteState, permutationOutputs: updated } };
            }
            default:
              return prev;
          }
        });
      },
      (success, error) => {
        if (!success && error) {
          toast({ description: error, status: 'error', duration: 5000, isClosable: true });
        } else {
          toast({ description: `${expectedCount} files permuted!`, status: 'success', duration: 5000, isClosable: true });
        }
        setState(prev => ({ permuteState: { ...prev.permuteState, processing: false } }));
      },
    );
  }, [toast, files, permutations]);

  const reverseFile = useCallback((file: string) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, processing: true } }));
    bridge.reverseFile(refreshState, (pState) => setState({ permuteState: pState }), file);
  }, [refreshState]);

  const trimFile = useCallback((file: string) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, processing: true } }));
    bridge.trimFile(refreshState, (pState) => setState({ permuteState: pState }), file);
  }, [refreshState]);

  const cancelProcessing = useCallback(() => {
    bridge.cancel();
    refreshState();
  }, [refreshState]);

  // ─── Simple setters (optimistic update, no re-fetch needed) ─────────────────

  const setDepth = useCallback(async (depth: number) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, permutationDepth: depth } }));
    await bridge.setDepth(depth);
  }, []);

  const setPermutations = useCallback(async (permutations: number) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, permutations } }));
    await bridge.setPermutations(permutations);
  }, []);

  const setNormalised = useCallback(async (normaliseAtEnd: boolean) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, normaliseAtEnd } }));
    await bridge.setNormalised(normaliseAtEnd);
  }, []);

  const setTrimAll = useCallback(async (trimAll: boolean) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, trimAll } }));
    await bridge.setTrimAll(trimAll);
  }, []);

  const setInputTrail = useCallback(async (inputTrail: number) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, inputTrail } }));
    await bridge.setInputTrail(inputTrail);
  }, []);

  const setOutputTrail = useCallback(async (outputTrail: number) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, outputTrail } }));
    await bridge.setOutputTrail(outputTrail);
  }, []);

  const setMaxStretch = useCallback(async (maxStretch: number) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, maxStretch } }));
    await bridge.setMaxStretch(maxStretch);
  }, []);

  const setCreateSubdirectories = useCallback(async (createSubfolders: boolean) => {
    setState(prev => ({ permuteState: { ...prev.permuteState, createSubdirectories: createSubfolders } }));
    await bridge.setCreateSubdirectories(createSubfolders);
  }, []);

  // ─── Processor pool ─────────────────────────────────────────────────────────
  // Optimistic updates so the UI responds immediately without waiting for IPC.

  const setProcessorEnabled = useCallback((name: string, enable: boolean) => {
    setState(prev => ({
      permuteState: {
        ...prev.permuteState,
        processorPool: enable
          ? [...prev.permuteState.processorPool, name]
          : prev.permuteState.processorPool.filter(p => p !== name),
      },
    }));
    if (enable) bridge.addProcessor(name);
    else bridge.removeProcessor(name);
  }, []);

  // Keep allProcessors in a ref so selectAll/deselectAll stay stable.
  const allProcessorsRef = useRef(allProcessors);
  allProcessorsRef.current = allProcessors;

  const selectAllProcessors = useCallback(async () => {
    setState(prev => ({ permuteState: { ...prev.permuteState, processorPool: [...allProcessorsRef.current] } }));
    await bridge.selectAllProcessors();
  }, []);

  const deselectAllProcessors = useCallback(async () => {
    setState(prev => ({ permuteState: { ...prev.permuteState, processorPool: [] } }));
    await bridge.deselectAllProcessors();
  }, []);

  // ─── File management ────────────────────────────────────────────────────────

  const addFiles = useCallback(async (files: string[]) => {
    await Promise.all(files.map((f) => bridge.addFile(f)));
    const permuteState = await bridge.getState();
    setState({ permuteState });
  }, []);

  const removeFile = useCallback(async (file: string) => {
    await bridge.removeFile(file);
    const permuteState = await bridge.getState();
    setState({ permuteState });
  }, []);

  const clearAllFiles = useCallback(async () => {
    await bridge.clearAllFiles();
    const permuteState = await bridge.getState();
    setState({ permuteState });
  }, []);

  const showFile = useCallback((file: string) => { bridge.showFile(file); }, []);

  const deleteOutputFile = useCallback(async (file: string) => {
    await bridge.deleteOutputFile(file);
    const permuteState = await bridge.getState();
    setState({ permuteState });
  }, []);

  const deleteAllOutputFiles = useCallback(async () => {
    await bridge.deleteAllOutputFiles();
    const permuteState = await bridge.getState();
    setState({ permuteState });
  }, []);

  const setOutput = useCallback(async () => {
    const chosen = await bridge.openOutputDialog();
    if (chosen) refreshState();
  }, [refreshState]);

  // ─── Scene ──────────────────────────────────────────────────────────────────

  const handleSaveScene = useCallback(async () => {
    const filePath = await bridge.saveScene();
    if (filePath) {
      toast({ description: 'Scene saved successfully', status: 'success', duration: 3000, isClosable: true });
      refreshState();
    }
  }, [toast, refreshState]);

  const handleLoadScene = useCallback(async () => {
    const response = await bridge.loadScene();
    if (response.success) {
      const permuteState = await bridge.getState();
      setState({ permuteState });
      toast({ description: 'Scene loaded successfully', status: 'success', duration: 3000, isClosable: true });
    } else {
      toast({ description: 'Error loading scene', status: 'error', duration: 5000, isClosable: true });
    }
  }, [toast]);

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
        processing={processing}
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
