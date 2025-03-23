import { Grid, GridItem, Heading, IconButton, Menu, MenuButton, MenuList, MenuItem, Box } from '@chakra-ui/react';
import { Processor } from './Processor';
import { memo, useCallback } from 'react';
import { LargeEllipsisIcon } from './icons/EllipsisIcon';
import { HamburgerIcon, LargeHamburgerIcon } from './icons/HamburgerIcon';

export interface IProcessorsProps {
  allProcessors: string[];
  processorPool: string[];
  setProcessorEnabled: (processor: string, enabled: boolean) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
}

export const Processors = memo(({
  allProcessors,
  processorPool,
  setProcessorEnabled,
  onSelectAll,
  onDeselectAll,
}: IProcessorsProps) => {
  const handleProcessorClick = useCallback((processor: string, enabled: boolean) => {
    setProcessorEnabled(processor, !enabled);
  }, [setProcessorEnabled]);

  const processors = allProcessors.map((ap) => {
    const enabled = processorPool.some((pp) => pp === ap);
    return (
      <Processor
        key={ap}
        name={ap}
        enabled={enabled}
        onClick={() => handleProcessorClick(ap, enabled)}
      />
    );
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
      <Box display="flex" alignItems="center" justifyContent="center" position="relative">
        <Heading textAlign="center" size="lg" color="brand.5600">
          Processors
        </Heading>
        <Menu>
          <MenuButton
            as={IconButton}
            aria-label="Processor options"
            icon={<LargeHamburgerIcon />}
            variant="ghost"
            rounded="full"
            size="sm"
            color="brand.5600"
            position="absolute"
            right="0"
            _hover={{ bg: 'brand.675' }}
          />
          <MenuList>
            <MenuItem onClick={onSelectAll}>
              Select All Processors
            </MenuItem>
            <MenuItem onClick={onDeselectAll}>
              Deselect All Processors
            </MenuItem>
          </MenuList>
        </Menu>
      </Box>
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
