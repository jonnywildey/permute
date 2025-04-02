import { Grid, GridItem, Heading, IconButton, Menu, MenuButton, MenuList, MenuItem, Box, Text, VStack } from '@chakra-ui/react';
import { Processor } from './Processor';
import { memo, useCallback } from 'react';
import { LargeHamburgerIcon } from './icons/HamburgerIcon';
import { processorCategories } from './processorDescriptions';

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
  debugger;

  const renderProcessorGroup = (category: string, processors: string[]) => {
    const categoryProcessors = processors.filter(p => allProcessors.includes(p)).map((ap) => {
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

    if (categoryProcessors.length === 0) return null;

    return (
      <VStack key={category} align="stretch" spacing={2} mb={2}>
        <Text fontSize="md" fontWeight="semibold" color="brand.200" ml={1} textAlign="left">
          {category}
        </Text>
        <Grid
          templateColumns="repeat(4, 1fr)"
          gap={3}
        >
          {categoryProcessors}
        </Grid>
      </VStack>
    );
  };

  return (
    <GridItem
      rowSpan={19}
      colSpan={6}
      maxHeight="100%"
      padding="4"
      overflow="hidden"
      overflowY="scroll"
    >
      <Box display="flex" alignItems="center" justifyContent="center" position="relative" mb={0}>
        <Heading textAlign="center" size="lg" color="brand.5600" pb={0} mb={0}>
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
      {Object.entries(processorCategories).map(([category, processors]) =>
        renderProcessorGroup(category, processors)
      )}
    </GridItem>
  );
});
