import { Grid, GridItem, Heading } from '@chakra-ui/react';
import { Processor } from './Processor';
import { memo, useCallback } from 'react';

export interface IProcessorsProps {
  allProcessors: string[];
  processorPool: string[];
  setProcessorEnabled: (processor: string, enabled: boolean) => void;
}

export const Processors = memo(({
  allProcessors,
  processorPool,
  setProcessorEnabled,
}: IProcessorsProps) => {
  const handleProcessorClick = useCallback((processor: string, enabled: boolean) => {
    setProcessorEnabled(processor, !enabled);
  }, [setProcessorEnabled]);

  const processors = allProcessors.map((ap) => {
    const enabled = processorPool.some((pp) => pp === ap);
    return <Processor key={ap} name={ap} enabled={enabled} onClick={() => handleProcessorClick(ap, enabled)} />;
  });

  return (
    <GridItem
      rowSpan={17}
      colSpan={6}
      maxHeight="100%"
      padding="4"
      overflow="hidden"
      overflowY="scroll"
    >
      <Heading textAlign="center" size="lg" color="brand.5600">
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
});
