import { GridItem, Button } from "@chakra-ui/react";
import type { IPermutationOutput } from "permute-node";

export interface IBottomBarProps {
  runProcessor: () => void;
  finished: boolean;
  permutationOutputs: IPermutationOutput[];
}

export const BottomBar: React.FC<IBottomBarProps> = ({ permutationOutputs, runProcessor, finished }) => {

  const progress = permutationOutputs[permutationOutputs.length - 1]?.progress;
  return (
  <>
  <GridItem rowSpan={2} colSpan={9} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" />
    <GridItem rowSpan={2} colSpan={3} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200">
         <Button onClick={runProcessor} disabled={!finished}>{finished ? "Run" : progress}</Button>
      </GridItem>
  </>
  );
}