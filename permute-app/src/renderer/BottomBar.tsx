import { GridItem, Button, CircularProgress, Slider, SliderTrack, SliderFilledTrack, SliderThumb, SliderMark, Switch, Heading, Grid } from "@chakra-ui/react";
import type { IPermutationOutput } from "permute-node";

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
  files: string[];
  setDepth: (depth: number) => void;
  setPermutations: (permutations: number) => void;
  setNormalised: (normaliseAtEnd: boolean) => void;
  setInputTrail: (trail: number) => void;
  setOutputTrail: (trail: number) => void;
}

export const BottomBar: React.FC<IBottomBarProps> = ({
  permutationOutputs, runProcessor, processing, setDepth, setInputTrail,
  setNormalised, setOutputTrail, setPermutations, depth, inputTrail, files, output,
  normaliseAtEnd, outputTrail, permutations
}) => {
  return (
    <GridItem rowSpan={5} colSpan={12}
      bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200"
      pt={3}
      pb={3}
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
    <Heading size="sm" textAlign="center">Input Trail (sec)</Heading>
    <Slider aria-label='slider-ex-2'
      min={0} max={8} step={1}
      colorScheme='pink' value={inputTrail} onChange={setInputTrail}
    >
      <SliderMark value={0} mt='2' ml='-0.75' fontSize='sm'>
        0
      </SliderMark>
      <SliderMark value={1} mt='2' ml='-0.75' fontSize='sm'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75' fontSize='sm'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75' fontSize='sm'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75' fontSize='sm'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75' fontSize='sm'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75' fontSize='sm'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75' fontSize='sm'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75' fontSize='sm'>
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
    <Heading size="sm" textAlign="center">Depth</Heading>
    <Slider aria-label='slider-ex-2'
      min={1} max={4} step={1}
      colorScheme='pink' value={depth} onChange={setDepth}
    >
      <SliderMark value={1} mt='2' ml='-0.75' fontSize='sm'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75' fontSize='sm'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75' fontSize='sm'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75' fontSize='sm'>
        4
      </SliderMark>
      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <SliderThumb />
    </Slider>
  </GridItem>
}

function Permutations(permutations: number, setPermutations: (permutations: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} pl={4} pt={3}>
    <Heading size="sm" textAlign="center">Permutations</Heading>
    <Slider aria-label='slider-ex-2'
      min={1} max={9} step={1}
      colorScheme='pink' value={permutations} onChange={setPermutations}
    >
      <SliderMark value={1} mt='2' ml='-0.75' fontSize='sm'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75' fontSize='sm'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75' fontSize='sm'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75' fontSize='sm'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75' fontSize='sm'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75' fontSize='sm'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75' fontSize='sm'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75' fontSize='sm'>
        8
      </SliderMark>
      <SliderMark value={9} mt='2' ml='-0.75' fontSize='sm'>
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
    <Heading size="sm" textAlign="center">Output Trail (sec)</Heading>
    <Slider aria-label='slider-ex-2'
      min={0} max={8} step={1}
      colorScheme='pink' value={outputTrail} onChange={setOutputTrail}
    >
      <SliderMark value={0} mt='2' ml='-0.75' fontSize='sm'>
        0
      </SliderMark>
      <SliderMark value={1} mt='2' ml='-0.75' fontSize='sm'>
        1
      </SliderMark>
      <SliderMark value={2} mt='2' ml='-0.75' fontSize='sm'>
        2
      </SliderMark>
      <SliderMark value={3} mt='2' ml='-0.75' fontSize='sm'>
        3
      </SliderMark>
      <SliderMark value={4} mt='2' ml='-0.75' fontSize='sm'>
        4
      </SliderMark>
      <SliderMark value={5} mt='2' ml='-0.75' fontSize='sm'>
        5
      </SliderMark>
      <SliderMark value={6} mt='2' ml='-0.75' fontSize='sm'>
        6
      </SliderMark>
      <SliderMark value={7} mt='2' ml='-0.75' fontSize='sm'>
        7
      </SliderMark>
      <SliderMark value={8} mt='2' ml='-0.75' fontSize='sm'>
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
  files: string[];
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
      bg="primary"
      fontSize="2xl"
    >
      {!processing ? "Run" : <CircularProgress value={progress} size={8} />}
    </Button>
  </GridItem>;
}

function Normalise(normaliseAtEnd: boolean, setNormalised: (normaliseAtEnd: boolean) => void) {
  return <GridItem rowSpan={1} colSpan={2} pl="33%">
    <Heading size="sm" fr>Normalise</Heading>
    <Switch fr isChecked={normaliseAtEnd} onChange={(e) => setNormalised(e.target.checked)} ml={2} />
  </GridItem>;
}
