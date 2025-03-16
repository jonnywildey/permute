
import {
  ChakraProvider,
  Grid,
  useDisclosure,
  useToast,
  Spinner,
  Center,
  Text,
  Heading
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

const Content = () => {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const toast = useToast();
  const { isOpen, onOpen, onClose } = useDisclosure({
    defaultIsOpen: !state.permuteState.output,
  });

  const {
    allProcessors,
    permutationDepth,
    files,
    output,
    permutations,
    normaliseAtEnd,
    inputTrail,
    outputTrail,
    processorPool,
    permutationOutputs,
  } = state.permuteState;
  const refreshState = async () => {
    const permuteState = await window.Electron.ipcRenderer.getState();
    setState({ ...state, permuteState });
  };
  useEffect(() => {
    const setup = async () => {
      const permuteState: IPermuteState =
        await window.Electron.ipcRenderer.getState();
      setState({
        permuteState,
      });
    };
    setup();
  }, []);

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
        const description = `${
          pState.files.length * pState.permutations
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
  const setOutput = async () => {
    window.Electron.ipcRenderer.openOutputDialog(([output]: [string]) => {
      window.Electron.ipcRenderer.setOutput(output);
      refreshState();
    });
  };
  const openWelcome = () => {
    onOpen();
  };

  const setProcessorEnabled = (name: string, enable: boolean) => {
    if (enable) {
      window.Electron.ipcRenderer.addProcessor(name);
    } else {
      window.Electron.ipcRenderer.removeProcessor(name);
    }
    refreshState();
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
      <Welcome isOpen={isOpen} onClose={onClose} />
      <TopBar openWelcome={openWelcome} />
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
      />
      <BottomBar
        permutationOutputs={permutationOutputs}
        runProcessor={runProcessor}
        processing={state.permuteState.processing}
        depth={permutationDepth}
        permutations={permutations}
        normaliseAtEnd={normaliseAtEnd}
        inputTrail={inputTrail}
        outputTrail={outputTrail}
        setDepth={setDepth}
        setPermutations={setPermutations}
        setNormalised={setNormalised}
        setInputTrail={setInputTrail}
        setOutputTrail={setOutputTrail}
        processorPool={processorPool}
        files={files}
        output={output}
      />
    </Grid>
  );
};

export default function App() {
  const [loading, setLoading] = useState(true);
  useEffect(() => {
    setTimeout(() => setLoading(false), 2000);
  }, []);
  return (
    <ChakraProvider theme={theme}>
      <CreateAudioContext>
        {loading ? (
          <>
            <div className="font_preload" style={{"opacity": 0}}>
              <Text>ABC</Text>
              <Heading>ABCDEFG</Heading>
            </div>
          <Center width="100vw" height="100vh">
            <Spinner ml={2} size="xl" color="brand.600" />
          </Center>
          </>
        ) : (
          <Content />
        )}
      </CreateAudioContext>
    </ChakraProvider>
  );
}
