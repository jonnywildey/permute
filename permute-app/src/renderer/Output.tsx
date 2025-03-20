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
} from '@chakra-ui/react';
import { ViewIcon, DeleteIcon } from '@chakra-ui/icons';
import type { IPermutationOutput } from 'permute-node';
import { useContext } from 'react';
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

const buttonBg = 'brand.500';
const bg = 'brand.25';
const fileBorderColour = 'brand.150';

export const Output: React.FC<IOutputProps> = ({
  output,
  showFile,
  reverseFile,
  setOutput,
  trimFile,
  permutationOutputs,
  deleteOutputFile,
}) => {
  const { playFile } = useContext(AudioContext);
  const outputBoxes = permutationOutputs
    .filter((f) => f.progress === 100 && f.image)
    .map((file) => {
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
            justifyContent="space-between"
          >
            <Heading
              size="sm"
              width="80%"
              display="inline"
              color="gray.600"
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
                size="xs"
                icon={<DeleteIcon />}
                onClick={() => deleteOutputFile(file.path)}
                color="gray.500"
                paddingTop={0}
                _hover={{ color: 'red.400' }}
                mr={2}
                mt="-5px"
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
          <Box display="flex" alignItems="baseline" width="100%" pos="relative">
            <Tooltip
              openDelay={200}
              label="Preview"
            >
              <IconButton
                aria-label="show"
                variant="ghost"
                size="xs"
                disabled={isAiff}
                icon={<PlayIcon />}
                onClick={() => playFile(file)}
              />
            </Tooltip>
            <Tooltip
              openDelay={200}
              label="Open directory"
            >
              <IconButton
                aria-label="show"
                variant="ghost"
                alignSelf="center"
                size="xs"
                icon={<ViewIcon />}
                onClick={() => showFile(file.path)}
              />
            </Tooltip>
            <Tooltip
              openDelay={200}
              label="Reverse"
            >
              <IconButton
                aria-label="show"
                variant="ghost"
                alignSelf="center"
                size="xs"
                icon={<ReverseIcon />}
                onClick={() => reverseFile(file.path)}
              />
            </Tooltip>
            <Tooltip
              openDelay={200}
              label="Auto-trim"
            >
              <IconButton
                aria-label="show"
                variant="ghost"
                alignSelf="center"
                size="xs"
                icon={<TrimIcon />}
                onClick={() => trimFile(file.path)}
              />
            </Tooltip>
            <Text
              pr={2}
              width="100%"
              textAlign="right"
              color="gray.500"
              fontSize="sm"
              lineHeight={1}
            >
              {displayTime(file.durationSec)}
            </Text>
          </Box>
        </Box>
      );
    });

  const directory = output ? (
    <Box
      display="flex"
      padding={3}
      mt={5}
      alignItems="center"
      borderTop="1px"
      borderTopColor="gray.300"
      borderBottom="1px"
      borderBottomColor={outputBoxes.length ? 'gray.400' : 'gray.300'}
      color="gray.800"
    >
      <IconButton
        aria-label="show"
        variant="ghost"
        size="sm"
        icon={<ViewIcon />}
        onClick={() => showFile(output)}
      />
      <Heading ml={3} className="output-heading" size="sm">
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
      <Heading textAlign="center" size="lg" color="gray.600">
        Output
      </Heading>
      <Center>
        <Button
          mt={3}
          bg={buttonBg}
          width="75%"
          onClick={setOutput}
          color="gray.50"
          fontSize="2xl"
          cursor="pointer"
          shadow="base"
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
};
