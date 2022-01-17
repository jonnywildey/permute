import { Button, GridItem, Heading } from "@chakra-ui/react";

import { createPermuteProcessor } from "../../../permute-node";

export const Output: React.FC = () => {
  const onClick = async () => {
     window.Electron.ipcRenderer.openOutputDialog((arg) => {
      // eslint-disable-next-line no-console
      console.log(arg);

      createPermuteProcessor();
    });
      
  }
  
  return <GridItem rowSpan={9} colSpan={3} bg='tomato'>
    <Heading textAlign="center" size="lg">Output</Heading>

    <Button onClick={onClick}>Select Output</Button>
    
  </GridItem>

}