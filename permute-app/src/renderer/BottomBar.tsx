import { GridItem, Button } from "@chakra-ui/react";


export const BottomBar: React.FC = () => {

  const update = (state) => {
    console.log(state);
  }
  
  const onClick = async () => {
     window.Electron.ipcRenderer.runProcessor(update, update);
  }

  return (
  <>
  <GridItem rowSpan={2} colSpan={9} bg='purple' />
    <GridItem rowSpan={2} colSpan={3} bg='purple'>
         <Button onClick={onClick}>Run</Button>
      </GridItem>
  </>
  );
}