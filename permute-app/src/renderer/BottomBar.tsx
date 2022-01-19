import { GridItem, Button, CircularProgress } from "@chakra-ui/react";
import type { IPermutationOutput } from "permute-node";

export interface IBottomBarProps {
  runProcessor: () => void;
  finished: boolean;
  permutationOutputs: IPermutationOutput[];
}

export const BottomBar: React.FC<IBottomBarProps> = ({ permutationOutputs, runProcessor, finished }) => {

  const progress = permutationOutputs.reduce((acc, permutationOutput) => {
    return acc + permutationOutput.progress
  }, 0) / permutationOutputs.length;
  return (
  <>
  <GridItem rowSpan={2} colSpan={9} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" />
    <GridItem rowSpan={2} colSpan={3} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" display="flex">
         <Button 
           onClick={runProcessor} 
           disabled={!finished}
           width="100%"
           bg="primary"
           fontSize="xl"
           margin={5}
           padding={5}
           >
             {finished ? "Run" : <CircularProgress value={progress} size={8} />}
          </Button>
      </GridItem>
  </>
  );
}