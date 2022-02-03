import { Button } from "@chakra-ui/react";


export interface IProcessorProps {
  name: string;
  enabled: boolean;
  onClick: () => void;
}

const bg = "brand.600";
const bdDisabled = "brand.650"

export const Processor: React.FC<IProcessorProps> = ({
  enabled, name, onClick
}) => (
  <Button 
    key={name}
    bg={enabled ? bg : bdDisabled} 
    color="gray.700"
    onClick={onClick} 
    width="100%"
    shadow="sm"
    borderRadius={20}
    >{name}</Button>
) 