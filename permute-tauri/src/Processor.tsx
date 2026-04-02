import { Button, Tooltip } from '@chakra-ui/react';
import { processorDescriptions } from './processorDescriptions';
import { memo, useCallback } from 'react';

const PROCESSOR_TOOLTIP_DELAY = 1400;

export interface IProcessorProps {
  name: string;
  enabled: boolean;
  onToggle: (name: string) => void;
}

const bg = 'brand.600';
const bdDisabled = 'brand.650';

export const Processor = memo(({
  enabled,
  name,
  onToggle,
}: IProcessorProps) => {
  // Stable click handler: name never changes for a given instance, and onToggle
  // is kept stable in Processors via the ref pattern, so memo bails out for all
  // processors except the one whose enabled state changed.
  const handleClick = useCallback(() => onToggle(name), [onToggle, name]);

  const button = (
    <Button
      key={name}
      bg={enabled ? bg : bdDisabled}
      className={enabled ? 'processor-enabled' : 'processor'}
      color="gray.700"
      onClick={handleClick}
      width="100%"
      shadow="sm"
      borderRadius={20}
      fontSize="lg"
      paddingTop="4px"
    >
      {name}
    </Button>
  );
  const description = processorDescriptions[name];
  if (description) {
    return (
      <Tooltip openDelay={PROCESSOR_TOOLTIP_DELAY} label={description} fontSize="md">
        {button}
      </Tooltip>
    );
  }

  return button;
});
