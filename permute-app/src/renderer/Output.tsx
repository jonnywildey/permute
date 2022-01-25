import { Box, Button, GridItem, Heading, Center, IconButton } from "@chakra-ui/react";
import { ViewIcon } from  "@chakra-ui/icons"

export interface IOutputProps {
  output: string;
  setOutput: () => void;
  showFile: (file: string) => void;
}

const buttonBg = "brand.500";
const bg = "brand.25";

export const Output: React.FC<IOutputProps> = ({ output, showFile, setOutput }) => {
  const directory = output ? 
  <Box 
    display="flex" 
    padding={3} 
    mt={5}
    alignItems="center" 
    borderTop="1px"
    borderTopColor="gray.300" 
    borderBottom="1px"
    borderBottomColor="gray.300" 
    color="gray.800"
    >
    <IconButton 
      aria-label="show" 
      variant="ghost"
      size="sm"
      icon={<ViewIcon  />} 
      onClick={() => showFile(output)} 
      />
    <Heading 
      ml={3}
      className="output-heading"
      size="sm" 
        >
        {output}
    </Heading>
  </Box> : null;
  return <GridItem rowSpan={17} colSpan={3} bg={bg} pt={4}>
    <Heading textAlign="center" size="lg" color="gray.600">Output</Heading>
    <Center>
    <Button mt={3} bg={buttonBg} width="75%" onClick={setOutput} color="gray.800">Select Output Directory</Button>
    </Center>
    {directory}
  </GridItem>

}