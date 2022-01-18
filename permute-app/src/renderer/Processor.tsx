import { Button } from "@chakra-ui/react";


export interface IProcessorProps {
  name: string;
  enabled: boolean;
  onClick: () => void;
}

export const Processor: React.FC<IProcessorProps> = ({
  enabled, name, onClick
}) => (
  <Button disabled={!enabled} onClick={onClick} >{name}</Button>
) 