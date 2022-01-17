import { GridItem, Button } from "@chakra-ui/react";


export const BottomBar: React.FC = () => {
  
  const onClick = () => {
    console.log("Clicked run");
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