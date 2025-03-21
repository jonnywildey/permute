import {
  GridItem,
  Button,
  CircularProgress,
  Slider,
  Text,
  SliderTrack,
  SliderFilledTrack,
  SliderThumb,
  SliderMark,
  Switch,
  Heading,
  Grid,
  Tooltip,
  useColorMode,
} from '@chakra-ui/react';
import type { IPermutationOutput, IPermutationInput } from 'permute-node';
import { AudioPlayer } from './AudioPlayer';
import { useState, useEffect } from 'react';

export interface IBottomBarProps {
  runProcessor: () => void;
  processing: boolean;
  permutationOutputs: IPermutationOutput[];
  depth: number;
  permutations: number;
  normaliseAtEnd: boolean;
  trimAll: boolean;
  inputTrail: number;
  outputTrail: number;
  output: string;
  files: IPermutationInput[];
  processorPool: string[];
  setDepth: (depth: number) => void;
  setPermutations: (permutations: number) => void;
  setNormalised: (normaliseAtEnd: boolean) => void;
  setTrimAll: (trimAll: boolean) => void;
  setInputTrail: (trail: number) => void;
  setOutputTrail: (trail: number) => void;
  cancelProcessing: () => void;
}

const buttonBg = 'brand.200';
const borderColour = 'gray.100';

export const BottomBar: React.FC<IBottomBarProps> = ({
  permutationOutputs,
  runProcessor,
  processing,
  setDepth,
  setInputTrail,
  setNormalised,
  setTrimAll,
  setOutputTrail,
  setPermutations,
  depth,
  inputTrail,
  files,
  output,
  normaliseAtEnd,
  trimAll,
  outputTrail,
  processorPool,
  permutations,
  cancelProcessing,
}) => {
  const { colorMode } = useColorMode();

  return (
    <GridItem
      rowSpan={6}
      colSpan={12}
      bg={colorMode === 'dark' ? 'brand.160' : 'brand.150'}
      // borderTop="0.5px solid"
      // borderTopColor={borderColour}
      color="brand.5700"
      borderRadius={20}
      shadow="md"
      minHeight="100%"
    >
      <Grid
        mt={3}
        mb={3}
        overflowY="auto"
        templateRows="repeat(2, 1fr)"
        templateColumns="repeat(12, 1fr)"
        width="100%"
        height="100%"
      >
        <AudioPlayer />
        {InputTrail(inputTrail, setInputTrail)}
        {Depth(depth, setDepth)}

        {Normalise(normaliseAtEnd, setNormalised)}
        <Run
          files={files}
          output={output}
          processing={processing}
          permutationOutputs={permutationOutputs}
          permutations={permutations}
          processorPool={processorPool}
          runProcessor={runProcessor}
          cancelProcessing={cancelProcessing}
        />
        {OutputTrail(outputTrail, setOutputTrail)}
        {Permutations(permutations, setPermutations)}
        {TrimAll(trimAll, setTrimAll)}
        <GridItem rowSpan={1} colSpan={2} />
      </Grid>
    </GridItem>
  );
};

function InputTrail(
  inputTrail: number,
  setInputTrail: (trail: number) => void
) {
  return (
    <GridItem rowSpan={1} colSpan={2} pr={4} pl={4}>
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            Adds extra seconds to the end of the audio file. Useful when using
            effects like delay with reverse
          </Text>
        }
      >
        <Heading size="md" textAlign="center">
          Start Trail (sec)
        </Heading>
      </Tooltip>
      <Slider
        aria-label="slider-ex-2"
        min={0}
        max={8}
        step={1}
        colorScheme="brand"
        value={inputTrail}
        onChange={setInputTrail}
        color="gray.5600"
        fontSize="sm"
      >
        <SliderMark value={0} mt="2" ml="-0.75">
          0
        </SliderMark>
        <SliderMark value={1} mt="2" ml="-0.75">
          1
        </SliderMark>
        <SliderMark value={2} mt="2" ml="-0.75">
          2
        </SliderMark>
        <SliderMark value={3} mt="2" ml="-0.75">
          3
        </SliderMark>
        <SliderMark value={4} mt="2" ml="-0.75">
          4
        </SliderMark>
        <SliderMark value={5} mt="2" ml="-0.75">
          5
        </SliderMark>
        <SliderMark value={6} mt="2" ml="-0.75">
          6
        </SliderMark>
        <SliderMark value={7} mt="2" ml="-0.75">
          7
        </SliderMark>
        <SliderMark value={8} mt="2" ml="-0.75">
          8
        </SliderMark>

        <SliderTrack>
          <SliderFilledTrack />
        </SliderTrack>
        <SliderThumb />
      </Slider>
    </GridItem>
  );
}

function Depth(depth: number, setDepth: (depth: number) => void) {
  return (
    <GridItem rowSpan={1} colSpan={2} pl={4}>
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            Controls how many processors the audio is run through. <br />
            High depth values can run up to 32 processors and can be noisy /
            very long. <br />
            Setting depth to 0 will always run the audio through 1 processor
          </Text>
        }
      >
        <Heading size="md" textAlign="center">
          Depth
        </Heading>
      </Tooltip>
      <Slider
        aria-label="slider-ex-2"
        min={0}
        max={4}
        step={1}
        colorScheme="brand"
        value={depth}
        onChange={setDepth}
        color="gray.5600"
        fontSize="sm"
      >
        <SliderMark value={0} mt="2" ml="-0.75">
          0
        </SliderMark>
        <SliderMark value={1} mt="2" ml="-0.75">
          1
        </SliderMark>
        <SliderMark value={2} mt="2" ml="-0.75">
          2
        </SliderMark>
        <SliderMark value={3} mt="2" ml="-0.75">
          3
        </SliderMark>
        <SliderMark value={4} mt="2" ml="-0.75">
          4
        </SliderMark>
        <SliderTrack>
          <SliderFilledTrack />
        </SliderTrack>
        <SliderThumb />
      </Slider>
    </GridItem>
  );
}

function Permutations(
  permutations: number,
  setPermutations: (permutations: number) => void
) {
  return (
    <GridItem rowSpan={1} colSpan={2} pl={4} pt={3}>
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            How many permutations to generate per file. <br />
            e.g. setting permutations to 5 and selecting one file will generate
            5 files <br />
            Selecting 2 files would generate 10
          </Text>
        }
      >
        <Heading size="md" textAlign="center">
          Permutations
        </Heading>
      </Tooltip>
      <Slider
        aria-label="slider-ex-2"
        min={1}
        max={9}
        step={1}
        colorScheme="brand"
        value={permutations}
        onChange={setPermutations}
        color="gray.5600"
        fontSize="sm"
      >
        <SliderMark value={1} mt="2" ml="-0.75">
          1
        </SliderMark>
        <SliderMark value={2} mt="2" ml="-0.75">
          2
        </SliderMark>
        <SliderMark value={3} mt="2" ml="-0.75">
          3
        </SliderMark>
        <SliderMark value={4} mt="2" ml="-0.75">
          4
        </SliderMark>
        <SliderMark value={5} mt="2" ml="-0.75">
          5
        </SliderMark>
        <SliderMark value={6} mt="2" ml="-0.75">
          6
        </SliderMark>
        <SliderMark value={7} mt="2" ml="-0.75">
          7
        </SliderMark>
        <SliderMark value={8} mt="2" ml="-0.75">
          8
        </SliderMark>
        <SliderMark value={9} mt="2" ml="-0.75">
          9
        </SliderMark>

        <SliderTrack>
          <SliderFilledTrack />
        </SliderTrack>
        <SliderThumb />
      </Slider>
    </GridItem>
  );
}

function OutputTrail(
  outputTrail: number,
  setOutputTrail: (trail: number) => void
) {
  return (
    <GridItem rowSpan={1} colSpan={2} pr={4} pt={3} pl={4}>
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            Adds extra seconds to the end of the audio file. Useful when using
            effects like delay
          </Text>
        }
      >
        <Heading size="md" textAlign="center">
          End Trail (sec)
        </Heading>
      </Tooltip>
      <Slider
        aria-label="slider-ex-2"
        min={0}
        max={8}
        step={1}
        colorScheme="brand"
        value={outputTrail}
        onChange={setOutputTrail}
        color="gray.5600"
        fontSize="sm"
      >
        <SliderMark value={0} mt="2" ml="-0.75">
          0
        </SliderMark>
        <SliderMark value={1} mt="2" ml="-0.75">
          1
        </SliderMark>
        <SliderMark value={2} mt="2" ml="-0.75">
          2
        </SliderMark>
        <SliderMark value={3} mt="2" ml="-0.75">
          3
        </SliderMark>
        <SliderMark value={4} mt="2" ml="-0.75">
          4
        </SliderMark>
        <SliderMark value={5} mt="2" ml="-0.75">
          5
        </SliderMark>
        <SliderMark value={6} mt="2" ml="-0.75">
          6
        </SliderMark>
        <SliderMark value={7} mt="2" ml="-0.75">
          7
        </SliderMark>
        <SliderMark value={8} mt="2" ml="-0.75">
          8
        </SliderMark>

        <SliderTrack>
          <SliderFilledTrack />
        </SliderTrack>
        <SliderThumb />
      </Slider>
    </GridItem>
  );
}

export interface IRunProps {
  runProcessor: () => void;
  processing: boolean;
  permutationOutputs: IPermutationOutput[];
  output: string;
  files: IPermutationInput[];
  processorPool: string[];
  permutations: number;
  cancelProcessing: () => void;
}

const Run: React.FC<IRunProps> = ({
  files,
  output,
  processing,
  runProcessor,
  permutationOutputs,
  processorPool,
  permutations,
  cancelProcessing,
}) => {
  const [timeElapsed, setTimeElapsed] = useState(0);
  const progress =
    permutationOutputs.reduce((acc, permutationOutput) => {
      return acc + permutationOutput.progress;
    }, 0) /
    (files.length * permutations);

  useEffect(() => {
    let interval: NodeJS.Timeout;
    if (processing) {
      interval = setInterval(() => {
        setTimeElapsed(prev => prev + 1);
      }, 1000);
    } else {
      setTimeElapsed(0);
    }
    return () => clearInterval(interval);
  }, [processing]);

  const isLongRunning = timeElapsed >= 5;
  const noFiles = files.length === 0;
  const noOutput = !output;
  const noProcessors = processorPool.length === 0;

  // During processing, only enable the button if it's been running long enough to show cancel
  // When not processing, disable if any required conditions are not met
  const isDisabled = processing
    ? !isLongRunning
    : (noFiles || noOutput || noProcessors);

  const getDisabledReason = () => {
    if (noFiles) return "Please add some audio files";
    if (noOutput) return "Please select an output directory";
    if (noProcessors) return "Please select at least one processor";
    return "";
  };

  return (
    <GridItem rowSpan={2} colSpan={3} display="flex" pl={6} pr={6} alignItems="center">
      <Tooltip label={!processing && isDisabled ? getDisabledReason() : ""} isDisabled={!isDisabled || processing}>
        <Button
          onClick={isLongRunning ? cancelProcessing : runProcessor}
          disabled={isDisabled}
          width="100%"
          bg={isDisabled ? "brand.210" : !processing ? buttonBg : undefined}
          color={!processing ? "gray.50" : undefined}
          fontSize="2xl"
          shadow="sm"
          _hover={isLongRunning ? { bg: "red.200" } : { bg: "brand.210" }}
          transition="all 0.3s ease-in-out"
          display="flex"
          alignItems="center"
          justifyContent={isLongRunning ? "flex-start" : "center"}
          gap={3}
          px={6}
          className={processing ? "color-shift" : ""}
          cursor={isDisabled ? "not-allowed" : "pointer"}
        >
          {!processing ? (
            'Run'
          ) : (
            <>
              <CircularProgress
                value={progress}
                color={isLongRunning ? "red.300" : "brand.300"}
                size={8}
                transition="all 2.3s cubic-bezier(0.4, 0, 0.2, 1)"
                className={isLongRunning ? "slide-in" : ""}
              />
              <span className={isLongRunning ? "slide-in" : ""}>
                {isLongRunning ? 'Cancel' : ''}
              </span>
            </>
          )}
        </Button>
      </Tooltip>
    </GridItem>
  );
};

function Normalise(
  normaliseAtEnd: boolean,
  setNormalised: (normaliseAtEnd: boolean) => void
) {
  return (
    <GridItem rowSpan={1} colSpan={2} pl="33%">
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            If enabled, normalises audio to ensure there is no digital clipping{' '}
            <b>(recommended)</b>
          </Text>
        }
      >
        <Heading size="md">Normalise</Heading>
      </Tooltip>
      <Switch
        colorScheme="brand"
        isChecked={normaliseAtEnd}
        onChange={(e) => setNormalised(e.target.checked)}
        ml={2}
      />
    </GridItem>
  );
}

function TrimAll(
  trimAll: boolean,
  setTrimAll: (trimAll: boolean) => void
) {
  return (
    <GridItem rowSpan={1} colSpan={2} pt={3} pl="33%">
      <Tooltip
        openDelay={200}
        label={
          <Text fontSize="lg">
            If enabled, trims all permuted audio, removing silence{' '}
          </Text>
        }
      >
        <Heading size="md">Trim All</Heading>
      </Tooltip>
      <Switch
        colorScheme="brand"
        isChecked={trimAll}
        onChange={(e) => setTrimAll(e.target.checked)}
        ml={2}
      />
    </GridItem>
  );
}
