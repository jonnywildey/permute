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

const Content = () => {
  const [state, setState] = useState<IPermuteState>({ files: [], permutationOutputs: [] } as any);

  const refreshState = async () => {
    const state = await window.Electron.ipcRenderer.getState();
    console.log(state);
    setState(state);
  };

  useEffect(() => { refreshState(); }, []);


  const runProcessor = async () => {
    const update = (state: IPermuteState) => {
      setState(state);
    }
     window.Electron.ipcRenderer.runProcessor(update);
  }

  return (
    <Box bgGradient='linear(to-r, green.200, pink.500)' w="100%" h="100vh">
      <Grid
  h='100vh'
  templateRows='repeat(12, 1fr)'
  templateColumns='repeat(12, 1fr)'
  gap={0}
>
    <TopBar />
      <Files files={state.files} refreshState={refreshState} />
  <GridItem rowSpan={9} colSpan={6}  bg='papayawhip' />
  <Output output={state.output} refreshState={refreshState}/>
  <BottomBar permutationOutputs={state.permutationOutputs} runProcessor={runProcessor} finished={state.finished}  />
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
