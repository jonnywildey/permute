import { Button } from "@chakra-ui/react";


export interface IProcessorProps {
  name: string;
  enabled: boolean;
  onClick: () => void;
}

export const Processor: React.FC<IProcessorProps> = ({
  enabled, name, onClick
}) => (
  <Button 
    key={name}
    bg={enabled ? "green.300" : "green.100"} 
    onClick={onClick} 
    width="100%"
    >{name}</Button>
) 