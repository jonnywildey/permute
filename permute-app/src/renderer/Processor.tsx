import { Button, Tooltip } from '@chakra-ui/react';
import { processorDescriptions } from './processorDescriptions';

export interface IProcessorProps {
  name: string;
  enabled: boolean;
  onClick: () => void;
}

const bg = 'brand.600';
const bdDisabled = 'brand.650';

export const Processor: React.FC<IProcessorProps> = ({
  enabled,
  name,
  onClick,
}) => {
  const button = (
    <Button
      key={name}
      bg={enabled ? bg : bdDisabled}
      className={enabled ? 'processor-enabled' : 'processor'}
      color="gray.700"
      onClick={onClick}
      width="100%"
      shadow="sm"
      borderRadius={20}
    >
      {name}
    </Button>
  );
  const description = processorDescriptions[name];
  if (description) {
    return (
      <Tooltip openDelay={500} label={description}>
        {button}
      </Tooltip>
    );
  }

  return button;
};
