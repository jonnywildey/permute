import {
  Box,
  Button,
  GridItem,
  Heading,
  Center,
  IconButton,
  PropsOf,
  Text,
  List,
  ListItem,
  Tooltip,
  useColorMode,
} from '@chakra-ui/react';
import { ViewIcon, DeleteIcon } from '@chakra-ui/icons';
import type { IPermutationOutput } from 'permute-node';
import { useContext, useCallback, memo } from 'react';
import { PlayIcon } from './icons/PlayIcon';
import { AudioContext } from './AudioContext';
import { displayTime } from './displayTime';
import { ReverseIcon } from './icons/ReverseIcon';
import { TrimIcon } from './icons/TrimIcon';

export interface IOutputProps {
  output: string;
  setOutput: () => void;
  permutationOutputs: IPermutationOutput[];
  showFile: (file: string) => void;
  reverseFile: (file: string) => void;
  trimFile: (file: string) => void;
  deleteOutputFile: (file: string) => void;
}

const buttonBg = 'brand.175';
const bg = 'brand.25';
const fileBorderColour = 'brand.150';

const OutputFile = memo(({ file, onDelete, onShow, onReverse, onTrim, onPlay }: {
  file: IPermutationOutput;
  onDelete: (path: string) => void;
  onShow: (path: string) => void;
  onReverse: (path: string) => void;
  onTrim: (path: string) => void;
  onPlay: (file: IPermutationOutput) => void;
}) => {
  const { colorMode } = useColorMode();
  const props: PropsOf<typeof Box> = {
    key: file.path,
    borderBottom: '1px solid',
    borderBottomColor: fileBorderColour,
    color: 'gray.700',
  };
  const ext = file.path.split(".").pop()?.toLowerCase();
  const isAiff = ext === "aif" || ext === "aiff"

  return (
    <Box {...props}>
      <Box
        pt={1}
        display="flex"
        alignItems="center"
        width="100%"
        pos="relative"
        marginBottom={1}
        justifyContent="space-between"
      >
        <Heading
          size="sm"
          width="80%"
          display="inline"
          color="brand.5600"
          pl={2}
        >
          {file.name}
        </Heading>
        <Tooltip
          openDelay={200}
          label="Delete file"
        >
          <IconButton
            aria-label="delete"
            variant="ghost"
            rounded="full"
            size="xs"
            icon={<DeleteIcon />}
            onClick={() => onDelete(file.path)}
            color="brand.5600"
            paddingTop={0}
            _hover={{ bg: 'brand.50', color: 'red.500' }}
            mt="-4px"
            marginRight={1}
          />
        </Tooltip>
      </Box>
      <Tooltip
        openDelay={200}
        label={
          <List>
            {file.processors.map((p, i) => (
              <ListItem key={`${p}${i}`}>
                {i + 1}: {p}
              </ListItem>
            ))}
          </List>
        }
      >
        <Box
          width="100%"
          className="output-image"
          pl={2}
          pr={2}
          mt="-4px"
          mb="-8px"
          dangerouslySetInnerHTML={{ __html: file.image }}
        />
      </Tooltip>
      <Box display="flex" alignItems="baseline" width="100%" pos="relative" marginTop={2}>
        <Tooltip
          openDelay={200}
          label="Preview"
        >
          <IconButton
            aria-label="show"
            variant="ghost"
            rounded="full"
            size="xs"
            disabled={isAiff}
            icon={<PlayIcon />}
            onClick={() => onPlay(file)}
            color="brand.5600"
            _hover={{ bg: 'brand.50' }}
          />
        </Tooltip>
        <Tooltip
          openDelay={200}
          label="Open directory"
        >
          <IconButton
            aria-label="show"
            variant="ghost"
            rounded="full"
            alignSelf="center"
            size="xs"
            icon={<ViewIcon />}
            onClick={() => onShow(file.path)}
            color="brand.5600"
            _hover={{ bg: 'brand.50' }}
          />
        </Tooltip>
        <Tooltip
          openDelay={200}
          label="Reverse"
        >
          <IconButton
            aria-label="show"
            variant="ghost"
            rounded="full"
            alignSelf="center"
            size="xs"
            icon={<ReverseIcon />}
            onClick={() => onReverse(file.path)}
            color="brand.5600"
            _hover={{ bg: 'brand.50' }}
          />
        </Tooltip>
        <Tooltip
          openDelay={200}
          label="Auto-trim"
        >
          <IconButton
            aria-label="show"
            variant="ghost"
            rounded="full"
            alignSelf="center"
            size="xs"
            icon={<TrimIcon />}
            onClick={() => onTrim(file.path)}
            color="brand.5600"
            _hover={{ bg: 'brand.50' }}
          />
        </Tooltip>
        <Text
          pr={2}
          width="100%"
          textAlign="right"
          fontSize="sm"
          lineHeight={1}
          color={colorMode === 'dark' ? 'brand.5600' : 'grey.500'}
        >
          {displayTime(file.durationSec)}
        </Text>
      </Box>
    </Box>
  );
});

export const Output = memo(({
  output,
  showFile,
  reverseFile,
  setOutput,
  trimFile,
  permutationOutputs,
  deleteOutputFile,
}: IOutputProps) => {
  const { playFile } = useContext(AudioContext);

  const handleDelete = useCallback((path: string) => {
    deleteOutputFile(path);
  }, [deleteOutputFile]);

  const handleShow = useCallback((path: string) => {
    showFile(path);
  }, [showFile]);

  const handleReverse = useCallback((path: string) => {
    reverseFile(path);
  }, [reverseFile]);

  const handleTrim = useCallback((path: string) => {
    trimFile(path);
  }, [trimFile]);

  const handlePlay = useCallback((file: IPermutationOutput) => {
    playFile(file);
  }, [playFile]);

  const outputBoxes = permutationOutputs
    .filter((f) => f.progress === 100 && f.image)
    .map((file) => (
      <OutputFile
        key={file.path}
        file={file}
        onDelete={handleDelete}
        onShow={handleShow}
        onReverse={handleReverse}
        onTrim={handleTrim}
        onPlay={handlePlay}
      />
    ));

  const directory = output ? (
    <Box
      display="flex"
      paddingTop={3}
      paddingBottom={3}
      paddingLeft={0}
      paddingRight={2}
      mt={5}
      alignItems="center"
      borderTop="1px"
      borderTopColor="gray.300"
      borderBottom="1px"
      borderBottomColor={outputBoxes.length ? 'gray.400' : 'gray.300'}
      color="brand.5800"
    >
      <Tooltip
        openDelay={200}
        label="Open directory"
      >
        <IconButton
          aria-label="show"
          variant="ghost"
          size="sm"
          icon={<ViewIcon />}
          onClick={() => showFile(output)}
          color="brand.5600"
          _hover={{ bg: 'brand.50' }}
        />
      </Tooltip>
      <Heading ml={1} className="output-heading" size="sm">
        {output}
      </Heading>
    </Box>
  ) : null;

  return (
    <GridItem
      rowSpan={17}
      colSpan={3}
      bg={bg}
      pt={4}
      borderRadius={20}
      shadow="base"
      display="flex"
      overflow="hidden"
      flexDirection="column"
    >
      <Heading textAlign="center" size="lg" color="brand.5600">
        Output
      </Heading>
      <Center>
        <Button
          mt={3}
          bg={buttonBg}
          width="75%"
          onClick={setOutput}
          color="gray.50"
          fontSize="xl"
          cursor="pointer"
          // shadow="base"
          _hover={{ bg: 'brand.150' }}
        >
          Select Output Directory
        </Button>
      </Center>
      {directory}
      <Box overflowY="scroll" overflowX="hidden">
        {outputBoxes}
      </Box>
    </GridItem>
  );
});
