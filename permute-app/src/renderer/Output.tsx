import { Button, GridItem, Heading } from "@chakra-ui/react";


export const Output: React.FC = () => {
  const onClick = async () => {
     window.Electron.ipcRenderer.openOutputDialog(([output]) => {
      window.Electron.ipcRenderer.setOutput(output);
    });
  }
  
  return <GridItem rowSpan={9} colSpan={3} bg='tomato'>
    <Heading textAlign="center" size="lg">Output</Heading>

    <Button onClick={onClick}>Select Output</Button>
    
  </GridItem>

}