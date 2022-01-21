import './App.css';
import { ChakraProvider, Box, Grid } from '@chakra-ui/react';
import { Files } from './Files';
import { TopBar } from './TopBar';
import { Output } from './Output';
import { BottomBar } from './BottomBar';
import { theme } from './theme';
import type { IPermuteState } from "permute-node";
import { useEffect, useState } from 'react';
import { Processors } from './Processors';

export interface IAppState {
  permuteState: IPermuteState;
  allProcessors: string[];
}

const defaultAppState: IAppState = {
  permuteState: { files: [], permutationOutputs: [], processorPool: [], } as any,
  allProcessors: [],
}

const Content = () => {
  const [state, setState] = useState<IAppState>(defaultAppState);
  const {
    permutationDepth,
    output,
    files,
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
      const permuteState: IPermuteState = await window.Electron.ipcRenderer.getState();
      setState({
        allProcessors: permuteState.processorPool,
        permuteState,
      });
    }
    setup();
  }, []);

  const runProcessor = async () => {
    window.Electron.ipcRenderer.runProcessor(refreshState);
  }
  const setDepth = async (depth: number) => {
    window.Electron.ipcRenderer.setDepth(depth);
    refreshState();
  }
  const setPermutations = async (permutations: number) => {
    window.Electron.ipcRenderer.setPermutations(permutations);
    refreshState();
  }
  const setNormalised = async (normaliseAtEnd: boolean) => {
    window.Electron.ipcRenderer.setNormalised(normaliseAtEnd);
    refreshState();
  }
  const setInputTrail = async (inputTrail: number) => {
    window.Electron.ipcRenderer.setInputTrail(inputTrail);
    refreshState();
  }
  const setOutputTrail = async (outputTrail: number) => {
    window.Electron.ipcRenderer.setOutputTrail(outputTrail);
    refreshState();
  }

  const setProcessorEnabled = (name: string, enable: boolean) => {
    if (enable) {
      window.Electron.ipcRenderer.addProcessor(name);
    } else {
      window.Electron.ipcRenderer.removeProcessor(name);
    }
    refreshState();
  }

  return (
    <Box w="100%" h="100vh">
      <Grid
        templateRows='repeat(24, 1fr)'
        templateColumns='repeat(12, 1fr)'
        gap={0}
      >
        <TopBar />

        <Files files={files} refreshState={refreshState} />
        <Processors allProcessors={state.allProcessors} processorPool={processorPool} setProcessorEnabled={setProcessorEnabled} />
        <Output output={output} refreshState={refreshState} />
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
          files={files}
          output={output}
        />
      </Grid>
    </Box>
  );
};

export default function App() {
  return (
    <ChakraProvider theme={theme}>
      <Content />
    </ChakraProvider>
  );
}
