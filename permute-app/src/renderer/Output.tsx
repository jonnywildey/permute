import { Box, Button, GridItem, Heading, Center, IconButton, PropsOf, Image, Text } from "@chakra-ui/react";
import { ViewIcon } from "@chakra-ui/icons"
import type { IPermutationOutput } from "permute-node";
import { PlayIcon } from "./PlayIcon";
import { useContext } from "react";
import { AudioContext } from "./AudioContext";
import { round } from "lodash";

export interface IOutputProps {
  output: string;
  setOutput: () => void;
  permutationOutputs: IPermutationOutput[];
  showFile: (file: string) => void;
}

const buttonBg = "brand.500";
const bg = "brand.25";
const fileBorderColour = "brand.150";

export const Output: React.FC<IOutputProps> = ({ output, showFile, setOutput, permutationOutputs }) => {
  const { playFile } = useContext(AudioContext);
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
        icon={<ViewIcon />}
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

  const outputBoxes = permutationOutputs
    .filter(f => f.progress === 100)
    .map((file, i) => {
      const props: PropsOf<typeof Box> = {
        key: file.path,
        borderBottom: "1px solid",
        borderBottomColor: fileBorderColour,
        color: "gray.700"
      };
      if (i === 0) {
        props.borderTop = "1px solid";
        props.borderTopColor = fileBorderColour;
      }
      return (<Box {...props}>
      <Box
        display="flex"
        alignItems="center"
        width="100%"
        pos="relative"
        justifyContent="space-between"
        >
      <Heading
        size="sm"
        width="80%"
        display="inline"
        color="gray.600"
        pl={2}
        >{file.name}</Heading>
        </Box>
        <Box 
          width="100%"
          pl={1}
          pr={1}
          mt="-4px"
          mb="-8px"
          dangerouslySetInnerHTML={{ __html: file.image }} 
          />
        <Box
        display="flex"
        alignItems="baseline"
        width="100%"
        pos="relative"
        >
        <IconButton
          aria-label="show"
          variant="ghost"
          size="xs"
          icon={<PlayIcon />}
          onClick={() => playFile(file.path)}
        />
        <IconButton
          aria-label="show"
          variant="ghost"
          size="xs"
          icon={<ViewIcon />}
          onClick={() => showFile(file.path)}
        />
        <Text pr={2} width="100%" textAlign="right" color="gray.500" fontSize="sm" lineHeight={1}>
          { round(file.durationSec, 2) }s
          {file.processors}
        </Text>
        </Box>
    </Box>);
    });

  return <GridItem rowSpan={17} colSpan={3} bg={bg} pt={4}>
    <Heading textAlign="center" size="lg" color="gray.600">Output</Heading>
    <Center>
      <Button mt={3} bg={buttonBg} width="75%" onClick={setOutput} color="gray.800">Select Output Directory</Button>
    </Center>
    {directory}
    <Box overflowY="scroll" height="310px">
      {outputBoxes}
    </Box>
  </GridItem>

}