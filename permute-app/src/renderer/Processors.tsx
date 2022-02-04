import { Grid, GridItem, Heading } from '@chakra-ui/react';
import { Processor } from './Processor';

export interface IProcessorsProps {
  allProcessors: string[];
  processorPool: string[];
  setProcessorEnabled: (processor: string, enabled: boolean) => void;
}

export const Processors: React.FC<IProcessorsProps> = ({
  allProcessors,
  processorPool,
  setProcessorEnabled,
}) => {
  const processors = allProcessors.map((ap) => {
    const enabled = processorPool.some((pp) => pp === ap);
    const onClick = () => setProcessorEnabled(ap, !enabled);

    return <Processor key={ap} name={ap} enabled={enabled} onClick={onClick} />;
  });
  return (
    <GridItem
      rowSpan={17}
      colSpan={6}
      maxHeight="100%"
      padding="4"
      overflow="hidden"
    >
      <Heading textAlign="center" size="lg" color="gray.600">
        Processors
      </Heading>
      <Grid
        templateRows={`repeat(${Math.floor(allProcessors.length / 3)}, 1fr)`}
        templateColumns="repeat(3, 1fr)"
        gap={5}
        pt={3}
      >
        {processors}
      </Grid>
    </GridItem>
  );
};
