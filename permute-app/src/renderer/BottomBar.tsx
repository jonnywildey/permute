import { GridItem, Button, CircularProgress, Slider, SliderTrack, SliderFilledTrack, SliderThumb, SliderMark, Switch, Heading } from "@chakra-ui/react";
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
  setDepth: (depth: number) => void;
  setPermutations: (permutations: number) => void;
  setNormalised: (normaliseAtEnd: boolean) => void;
  setInputTrail: (trail: number) => void;
  setOutputTrail: (trail: number) => void;
}

export const BottomBar: React.FC<IBottomBarProps> = ({
  permutationOutputs, runProcessor, processing, setDepth, setInputTrail,
  setNormalised, setOutputTrail, setPermutations, depth, inputTrail,
  normaliseAtEnd, outputTrail, permutations
}) => {

  const progress = permutationOutputs.reduce((acc, permutationOutput) => {
    return acc + permutationOutput.progress
  }, 0) / permutationOutputs.length;
  return (
    <>
      <GridItem rowSpan={2} colSpan={3} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" padding={3} />
      {InputTrail(inputTrail, setInputTrail)}
      {Depth(depth, setDepth)}

      {Normalise(normaliseAtEnd, setNormalised)}
      {Run(runProcessor, processing, progress)}
      {OutputTrail(outputTrail, setOutputTrail)}
      {Permutations(permutations, setPermutations)}
      <GridItem rowSpan={1} colSpan={2} bg='cyan.100' />

    </>
  );
}

function InputTrail(inputTrail: number, setInputTrail: (trail: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" pr={4} pt={2}>
    <Heading size="sm" textAlign="center">Input Trail (sec)</Heading>
    <Slider aria-label='slider-ex-2'
      min={1} max={9} step={1}
      colorScheme='pink' value={inputTrail} onChange={setInputTrail}
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

function Depth(depth: number, setDepth: (depth: number) => void) {
  return <GridItem rowSpan={1} colSpan={2} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" pl={4} pt={2}> 
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
  return <GridItem rowSpan={1} colSpan={2} bg='cyan.100' pt={3} pb={2} pl={4}>
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
  return <GridItem rowSpan={1} colSpan={2} bg='cyan.100' pt={3} pb={3} pr={4}>
    <Heading size="sm" textAlign="center">Output Trail (sec)</Heading>
    <Slider aria-label='slider-ex-2'
      min={1} max={9} step={1}
      colorScheme='pink' value={outputTrail} onChange={setOutputTrail}
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

function Run(runProcessor: () => void, processing: boolean, progress: number) {
  return <GridItem rowSpan={2} colSpan={3} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" display="flex" padding={3}>
    <Button
      onClick={runProcessor}
      disabled={processing}
      width="100%"
      bg="primary"
      fontSize="xl"
      margin={5}
      padding={5}
    >
      {!processing ? "Run" : <CircularProgress value={progress} size={8} />}
    </Button>
  </GridItem>;
}

function Normalise(normaliseAtEnd: boolean, setNormalised: (normaliseAtEnd: boolean) => void) {
  return <GridItem rowSpan={1} colSpan={2} bg='cyan.100' borderTop="0.5px solid" borderTopColor="cyan.200" pt={2} pl="33%">
    <Heading size="sm" fr>Normalise</Heading>
    <Switch  fr isChecked={normaliseAtEnd} onChange={(e) => setNormalised(e.target.checked)} ml={2} />
  </GridItem>;
}
