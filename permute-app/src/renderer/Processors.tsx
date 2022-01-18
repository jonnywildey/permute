import { GridItem } from "@chakra-ui/react";
import { Processor } from "./Processor";

export interface IProcessorsProps {
  allProcessors: string[];
  processorPool: string[];
  setProcessorEnabled: (processor: string, enabled: boolean) => void;
}

export const Processors: React.FC<IProcessorsProps> = ({ allProcessors, processorPool, setProcessorEnabled }) => {
    const processors = allProcessors.map( (ap) => {
    const enabled = processorPool.some( pp => pp === ap );
    const onClick = () => setProcessorEnabled(ap, !enabled);

    return <Processor
       name={ap}
       enabled={enabled}
       onClick={onClick}
    />
  });
  return (
    <GridItem rowSpan={9} colSpan={6}  bg='gray.50' >
      {processors}
      </GridItem>
  );
}