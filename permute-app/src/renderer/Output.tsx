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
  Link,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
  Portal,
} from '@chakra-ui/react';
import { ViewIcon, DeleteIcon } from '@chakra-ui/icons';
import type { IPermutationOutput } from 'permute-node';
import { useContext, useCallback, memo } from 'react';
import { PlayIcon } from './icons/PlayIcon';
import { AudioContext } from './AudioContext';
import { displayTime } from './displayTime';
import { ReverseIcon } from './icons/ReverseIcon';
import { TrimIcon } from './icons/TrimIcon';
import { LargeFolderIcon } from './icons/FolderIcon';
import { LargeTrashIcon } from './icons/TrashIcon';
import { InfoIcon } from './icons/InfoIcon';

export interface IOutputProps {
  output: string;
  setOutput: () => void;
  permutationOutputs: IPermutationOutput[];
  showFile: (file: string) => void;
  reverseFile: (file: string) => void;
  trimFile: (file: string) => void;
  deleteOutputFile: (file: string) => void;
  deleteAllOutputFiles: () => void;
}

const buttonBg = 'brand.175';
const bg = 'brand.25';
const fileBorderColour = 'brand.150';

const tooltipDelay = 600;

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
    borderBottom: '1px solid',
    borderBottomColor: fileBorderColour,
    color: 'gray.50',
  };
  const ext = file.path.split(".").pop()?.toLowerCase();
  const isAiff = ext === "aif" || ext === "aiff"

  return (
    <Box key={file.path} {...props}>
      <Box
        pt={1}
        display="flex"
        alignItems="center"
        width="100%"
        pos="relative"
        marginBottom={1}
        justifyContent="space-between"
      >
        <Tooltip
          openDelay={500}
          label={file.name}
        >
          <Heading
            size="sm"
            width="80%"
            display="block"
            color="brand.5600"
            pl={2}
            className="filename-ellipsis"
          >
            {file.name}
          </Heading>
        </Tooltip>
        <Tooltip
          openDelay={tooltipDelay}
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
            _hover={{ bg: 'brand.50', color: 'pink.700' }}
            mt="-4px"
            marginRight={1}
            boxSize="20px"
          />
        </Tooltip>
      </Box>
      <Box
        width="100%"
        className="output-image"
        pl={2}
        pr={2}
        mt="-4px"
        mb="-8px"
        dangerouslySetInnerHTML={{ __html: file.image }}
      />
      <Box display="flex" alignItems="baseline" width="100%" pos="relative" marginTop={2}>
        <Tooltip
          openDelay={tooltipDelay}
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
          openDelay={tooltipDelay}
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
          openDelay={tooltipDelay}
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
          openDelay={tooltipDelay}
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
        <Menu>
          <Tooltip label="Show processors" openDelay={tooltipDelay}>
            <MenuButton
              as={IconButton}
              aria-label="Show processors"
              icon={<InfoIcon />}
              variant="ghost"
              rounded="full"
              size="xs"
              color="brand.525"
              _hover={{ bg: 'brand.50' }}
            />
          </Tooltip>
          <Portal>
            <MenuList pl={4} pr={4} pt={2} pb={2}>
              <List spacing={1}>
                {file.processors.map((p: string, i: number) => (
                  <ListItem key={`${p}${i}`} fontSize="sm">
                    {i + 1}: {p}
                  </ListItem>
                ))}
              </List>
            </MenuList>
          </Portal>
        </Menu>
        <Text
          pr={2}
          width="100%"
          textAlign="right"
          fontSize="sm"
          lineHeight={1}
          color={colorMode === 'dark' ? 'brand.5600' : 'gray.500'}
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
  deleteAllOutputFiles,
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
    .filter((f) => f.progress === 100 && f.image && !f.deleted)
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

  const directory = output && (
    <Box
      display="flex"
      paddingTop={3}
      paddingBottom={3}
      paddingLeft={2}
      paddingRight={2}
      mt={5}
      alignItems="center"
      borderTop="1px"
      borderTopColor="brand.150"
      borderBottom="1px"
      borderBottomColor="brand.150"
      color="brand.5800"
    >
      <Box
        display="flex"
        color="brand.5600"
        mr={2}
        cursor="pointer"
        _hover={{ color: 'brand.5800' }}
      >
        <IconButton
          aria-label="show"
          variant="ghost"
          rounded="full"
          alignSelf="center"
          size="xs"
          icon={<LargeFolderIcon />}
          onClick={() => showFile(output)}
          color="brand.5600"
          _hover={{ bg: 'brand.50' }}
        />
      </Box>
      <Link
        onClick={() => showFile(output)}
        flex={1}
        className="output-heading"
        fontSize="sm"
        fontWeight="semibold"
        color="brand.5600"
        _hover={{ textDecoration: 'underline', color: 'brand.5800' }}
      >
        {output}
      </Link>
    </Box>
  );
  const completeFiles = permutationOutputs.filter((f) => f.progress === 100 && f.image && !f.deleted);
  const deleteAll = output && permutationOutputs.length > 0 && (
    <Box
      display="flex"
      alignItems="center"
      justifyContent="flex-end"
      px={2}
      py={1}
      borderBottom="1px"
      borderBottomColor="brand.150"
      fontSize="sm"
      color="brand.5600"
    >
      <Text justifyContent="start" mr={2} flex={1} fontSize="sm">
        {completeFiles.length} files
      </Text>
      <Tooltip openDelay={1000}
        label="Delete all permuted files">
        <Button
          variant="ghost"
          size="sm"
          hidden={completeFiles.length === 0}
          leftIcon={<LargeTrashIcon />}
          onClick={deleteAllOutputFiles}
          color="brand.5600"
          _hover={{ bg: 'brand.50', color: 'pink.700' }}
        >
          Delete All
        </Button>
      </Tooltip>
    </Box>
  )

  return (
    <GridItem
      rowSpan={19}
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
      {deleteAll}
      <Box overflowY="scroll" overflowX="hidden">
        {outputBoxes}
      </Box>
    </GridItem>
  );
});
