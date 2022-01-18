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
  <GridItem rowSpan={2} colSpan={9} bg='purple' />
    <GridItem rowSpan={2} colSpan={3} bg='purple'>
         <Button onClick={runProcessor} disabled={!finished}>{finished ? "Run" : progress}</Button>
      </GridItem>
  </>
  );
}