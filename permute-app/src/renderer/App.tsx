import { MemoryRouter as Router, Routes, Route } from 'react-router-dom';
// import icon from '../../assets/icon.svg';
import './App.css';
import { ChakraProvider, Box, extendTheme, Grid, GridItem } from '@chakra-ui/react';

const Hello = () => {
  return (
    <Box bgGradient='linear(to-r, green.200, pink.500)' w="100%" h="100vh">
      <Grid
  h='100vh'
  templateRows='repeat(12, 1fr)'
  templateColumns='repeat(12, 1fr)'
  gap={0}
>
    <TopBar />
      <Files />
  <GridItem rowSpan={9} colSpan={6}  bg='papayawhip' />
  <Output />
  <BottomBar />
</Grid>
    </Box>
  );
};


import { createBreakpoints } from '@chakra-ui/theme-tools'
import { Files } from './Files';
import { TopBar } from './TopBar';
import { Output } from './Output';
import { BottomBar } from './BottomBar';

const theme = extendTheme(

     createBreakpoints({
      sm: '1200em',
      md: '1200em',
      lg: '1200em',
      xl: '1200em',
      '2xl': '1200em',
  })
);

export default function App() {
  return (
    <ChakraProvider theme={theme}>
      <Router>
        <Routes>
          <Route path="/" element={<Hello />} />
        </Routes>
      </Router>
    </ChakraProvider>
  );
}
