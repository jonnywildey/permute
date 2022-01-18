// import icon from '../../assets/icon.svg';
import './App.css';
import { ChakraProvider, Box, Grid, GridItem } from '@chakra-ui/react';
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
  permuteState: { files: [], permutationOutputs: [] } as any,
  allProcessors: [],
}

const Content = () => {
  const [state, setState] = useState<IAppState>(defaultAppState);

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
  h='100vh'
  templateRows='repeat(12, 1fr)'
  templateColumns='repeat(12, 1fr)'
  gap={0}
>
    <TopBar />

      <Files files={state.permuteState.files} refreshState={refreshState} />
   <Processors allProcessors={state.allProcessors} processorPool={state.permuteState.processorPool} setProcessorEnabled={setProcessorEnabled} />
  <Output output={state.permuteState.output} refreshState={refreshState}/>
  <BottomBar permutationOutputs={state.permuteState.permutationOutputs} runProcessor={runProcessor} finished={state.permuteState.finished}  />
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
