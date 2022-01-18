import { Box, Button, GridItem, Heading } from "@chakra-ui/react";

export interface IOutputProps {
  output: string;
  refreshState: () => void;
}

export const Output: React.FC<IOutputProps> = ({ output, refreshState }) => {
  const onClick = async () => {
     window.Electron.ipcRenderer.openOutputDialog(([output]) => {
      window.Electron.ipcRenderer.setOutput(output);
      refreshState();
    });
  }
  
  return <GridItem rowSpan={9} colSpan={3} bg='red.50'>
    <Heading textAlign="center" size="lg">Output</Heading>

    <Button onClick={onClick}>Select Output</Button>

    <Box>{output}</Box>
    
  </GridItem>

}