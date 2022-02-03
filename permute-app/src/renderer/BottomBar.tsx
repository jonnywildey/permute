import { GridItem, Button, CircularProgress, Slider, Text,
  SliderTrack, SliderFilledTrack, SliderThumb, SliderMark, Switch, Heading, Grid, Tooltip 
} from "@chakra-ui/react";
import type { IPermutationOutput, IPermutationInput } from "permute-node";

export interface IBottomBarProps {
  runProcessor: () => void;
  processing: boolean;
  permutationOutputs: IPermutationOutput[];
  depth: number;
  permutations: number;
  normaliseAtEnd: boolean;
  inputTrail: number;
  outputTrail: number;
  output: string;
  files: IPermutationInput[];
  setDepth: (depth: number) => void;
  setPermutations: (permutations: number) => void;
  setNormalised: (normaliseAtEnd: boolean) => void;
  setInputTrail: (trail: number) => void;
  setOutputTrail: (trail: number) => void;
}

const bg = "brand.150";
const buttonBg = "brand.200";
const borderColour = "gray.100"

export const BottomBar: React.FC<IBottomBarProps> = ({
  permutationOutputs, runProcessor, processing, setDepth, setInputTrail,
  setNormalised, setOutputTrail, setPermutations, depth, inputTrail, files, output,
  normaliseAtEnd, outputTrail, permutations
}) => {
  return (
    <GridItem rowSpan={6} colSpan={12}
      bg={bg} borderTop="0.5px solid" borderTopColor={borderColour}
      pt={3}
      pb={3}
      color="gray.700"
              borderRadius={20}
                shadow="md"
    >
      <Grid
        templateRows={`repeat(2, 1fr)`}
        templateColumns='repeat(12, 1fr)'
        width="100%"
        height="100%"
      >
        <GridItem rowSpan={2} colSpan={3} padding={3} />
        {InputTrail(inputTrail, setInputTrail)}
        {Depth(depth, setDepth)}

        {Normalise(normaliseAtEnd, setNormalised)}
        <Run files={files} output={output} processing={processing} permutationOutputs={permutationOutputs} runProcessor={runProcessor} />
        {OutputTrail(outputTrail, setOutputTrail)}
        {Permutations(permutations, setPermutations)}
        <GridItem rowSpan={1} colSpan={2} />
      </Grid>
    </GridItem>
  );
}

function InputTrail(inputTrail: number, setInputTrail: (trail: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} pr={4}>
       <Tooltip openDelay={200} label={<Text fontSize="lg">
      Adds extra seconds to the end of the audio file. Useful when using effects like delay with reverse
    </Text>}>
    <Heading size="sm" textAlign="center">Start Trail (sec)</Heading>
    </Tooltip>
    <Slider aria-label='slider-ex-2'
      min={0} max={8} step={1}
      colorScheme='brand' value={inputTrail} onChange={setInputTrail}
            color="gray.600"
      fontSize='sm'
    >
      <SliderMark value={0} mt='2' ml='-0.75'>
        0
      </SliderMark>
      <SliderMark value={1} mt='2' ml='-0.75'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75'>
        8
      </SliderMark>

      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  </GridItem>;
}

function Depth(depth: number, setDepth: (depth: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} pl={4}>
    <Tooltip openDelay={200} label={<Text fontSize="lg">
      Controls how many processors the audio is run through.  <br />
      High depth values can run up to 32 processors and can be noisy
    </Text>}>
      <Heading size="sm" textAlign="center">Depth</Heading>
    </Tooltip>
    <Slider aria-label='slider-ex-2'
      min={1} max={4} step={1}
      colorScheme='brand' value={depth} onChange={setDepth}
            color="gray.600"
      fontSize='sm'
    >
      <SliderMark value={1} mt='2' ml='-0.75'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75'>
        4
      </SliderMark>
      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  </GridItem >
}

function Permutations(permutations: number, setPermutations: (permutations: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} pl={4} pt={3}>
    <Tooltip openDelay={200} label={<Text fontSize="lg">
      How many permutations to generate per file. <br />
      e.g. setting permutations to 5 and selecting one file will generate 5 files <br />
      Selecting 2 files would generate 10
    </Text>}>
      <Heading size="sm" textAlign="center">Permutations</Heading>
    </Tooltip>
    <Slider aria-label='slider-ex-2'
      min={1} max={9} step={1}
      colorScheme='brand' value={permutations} onChange={setPermutations}
      color="gray.600"
      fontSize='sm'
    >
      <SliderMark value={1} mt='2' ml='-0.75' >
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75'>
        8
      </SliderMark>
      <SliderMark value={9} mt='2' ml='-0.75'>
        9
      </SliderMark>

      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  </GridItem>;
}

function OutputTrail(outputTrail: number, setOutputTrail: (trail: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} pr={4} pt={3}>
    <Tooltip openDelay={200} label={<Text fontSize="lg">
      Adds extra seconds to the end of the audio file. Useful when using effects like delay
    </Text>}>
    <Heading size="sm" textAlign="center">End Trail (sec)</Heading>
    </Tooltip>
    <Slider aria-label='slider-ex-2'
      min={0} max={8} step={1}
      colorScheme='brand' value={outputTrail} onChange={setOutputTrail}
            color="gray.600"
      fontSize='sm'
    >
      <SliderMark value={0} mt='2' ml='-0.75'>
        0
      </SliderMark>
      <SliderMark value={1} mt='2' ml='-0.75'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75'>
        8
      </SliderMark>

      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  </GridItem>;
}

export interface IRunProps {
  runProcessor: () => void;
  processing: boolean;
  permutationOutputs: IPermutationOutput[];
  output: string;
  files: IPermutationInput[];
}

const Run: React.FC<IRunProps> = ({
  files,
  output,
  processing,
  runProcessor,
  permutationOutputs,
}) => {
  const progress = permutationOutputs.reduce((acc, permutationOutput) => {
    return acc + permutationOutput.progress
  }, 0) / permutationOutputs.length;
  return <GridItem rowSpan={2} colSpan={3} display="flex" pl={6} pr={6}>
    <Button
      onClick={runProcessor}
      disabled={processing || !output || !files.length}
      width="100%"
      bg={buttonBg}
      color="gray.50"
      fontSize="2xl"
      shadow="sm"
    >
      {!processing ? "Run" : <CircularProgress value={progress} size={8} />}
    </Button>
  </GridItem>;
}

function Normalise(normaliseAtEnd: boolean, setNormalised: (normaliseAtEnd: boolean) => void) {
  return <GridItem rowSpan={1} colSpan={2} pl="33%" >
    <Tooltip openDelay={200} label={<Text fontSize="lg">
      If enabled, normalises audio to ensure there is no digital clipping <b>(recommended)</b>
    </Text>}>
      <Heading size="sm">Normalise</Heading>
    </Tooltip>
    <Switch colorScheme="brand"
      isChecked={normaliseAtEnd} onChange={(e) => setNormalised(e.target.checked)} ml={2} />
  </GridItem>;
}
