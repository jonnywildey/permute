import { Button, GridItem, Heading, Input } from "@chakra-ui/react";


export const Output: React.FC = () => {
  const onClick = () => {
    debugger;
    const directory = window.electron.ipcRenderer.openOutputDialog();
    console.log(directory);
  }
  
  return <GridItem rowSpan={9} colSpan={3} bg='tomato'>
    <Heading textAlign="center" size="lg">Output</Heading>

    <Button onClick={onClick}>Select Output</Button>
    
  </GridItem>

}