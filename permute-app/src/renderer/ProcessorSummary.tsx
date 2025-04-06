import {
  Box,
  IconButton,
  List,
  ListItem,
  Tooltip,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
  Portal,
  Accordion,
  AccordionItem,
  AccordionButton,
  AccordionPanel,
  AccordionIcon,
} from '@chakra-ui/react';
import { InfoIcon } from './icons/InfoIcon';
import type { IProcessor } from 'permute-node';

const PROCESSOR_SUMMARY_TOOLTIP_DELAY = 2000;

interface ProcessorSummaryProps {
  processors: IProcessor[];
  tooltipDelay?: number;
}

export const ProcessorSummary = ({ processors, tooltipDelay = PROCESSOR_SUMMARY_TOOLTIP_DELAY }: ProcessorSummaryProps) => {
  return (
    <Menu lazyBehavior='unmount'>
      <Tooltip label="Show processors" openDelay={tooltipDelay}>
        <MenuButton
          as={IconButton}
          aria-label="Show processors"
          icon={<InfoIcon />}
          variant="ghost"
          rounded="full"
          size="xs"
          color="brand.525"
          _hover={{ bg: 'brand.50' }}
        />
      </Tooltip>
      <Portal>
        <MenuList pl={4} pr={4} pt={2} pb={2} maxH="250px" width="300px" overflowY="auto">
          <List spacing={1}>
            {processors.map((p: IProcessor, i: number) => (
              <ListItem key={`${p.name}${i}`} fontSize="md">
                {p.attributes.length > 0 ? (
                  <Accordion allowToggle>
                    <AccordionItem border="none">
                      <AccordionButton p={1} _hover={{ bg: 'brand.50' }}>
                        <Box flex="1" textAlign="left">
                          {i + 1}: {p.name}
                        </Box>
                        <AccordionIcon />
                      </AccordionButton>
                      <AccordionPanel pb={2} pl={0} pt={0}>
                        <Box width="100%" fontSize="sm">
                          <Box
                            display="flex"
                            borderBottom="1px"
                            borderColor="brand.150"
                            py={0}
                            mb={1}
                            fontWeight="medium"
                          />
                          {p.attributes.map((a) => (
                            <Box
                              key={a.key}
                              display="flex"
                              borderBottom="1px"
                              borderColor="brand.150"
                              py={1}
                            >
                              <Box
                                flex="0 0 40%"
                                color="brand.5600"
                                fontWeight="medium"
                                pr={2}
                                overflow="hidden"
                                textOverflow="ellipsis"
                                whiteSpace="nowrap"
                              >
                                {a.key}
                              </Box>
                              <Box
                                flex="1"
                                color="brand.5600"
                                overflow="hidden"
                                textOverflow="ellipsis"
                                whiteSpace="nowrap"
                              >
                                {a.value}
                              </Box>
                            </Box>
                          ))}
                        </Box>
                      </AccordionPanel>
                    </AccordionItem>
                  </Accordion>
                ) : (
                  <MenuItem
                    p={1}
                    _hover={{ bg: 'brand.50' }}
                    color="brand.5600"
                    fontSize="md"
                    cursor="default"
                    closeOnSelect={false}
                    onClick={(e) => {
                      e.stopPropagation();
                    }}
                  >
                    {i + 1}: {p.name}
                  </MenuItem>
                )}
              </ListItem>
            ))}
          </List>
        </MenuList>
      </Portal>
    </Menu>
  );
}; 