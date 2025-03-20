import {
  Box,
  GridItem,
  Heading,
  Input,
  IconButton,
  CloseButton,
  PropsOf,
  Button,
  Center,
  Text,
  Tooltip,
} from '@chakra-ui/react';
import { ViewIcon } from '@chakra-ui/icons';
import { useContext, useCallback, memo } from 'react';
import type { IPermutationInput } from 'permute-node';
import { AudioContext } from './AudioContext';
import { PlayIcon } from './icons/PlayIcon';
import { displayTime } from './displayTime';
import { useDropzone } from 'react-dropzone'

export interface IFilesProps {
  files: IPermutationInput[];
  addFiles: (files: string[]) => void;
  removeFile: (file: string) => void;
  showFile: (file: string) => void;
}

const buttonBg = 'brand.500';
const bg = 'brand.25';
const fileBorderColour = 'brand.150';

const FileBox = memo(({ file, onRemove, onShow, onPlay }: {
  file: IPermutationInput;
  onRemove: (path: string) => void;
  onShow: (path: string) => void;
  onPlay: (file: IPermutationInput) => void;
}) => {
  const props: PropsOf<typeof Box> = {
    key: file.path,
    borderBottom: '1px solid',
    borderBottomColor: fileBorderColour,
    pos: 'relative',
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
          color="gray.600"
          pl={2}
        >
          {file.name}
        </Heading>
        <CloseButton
          display="inline"
          float="right"
          color="gray.600"
          size="sm"
          onClick={() => onRemove(file.path)}
        />
      </Box>
      <Box
        width="100%"
        mt="-4px"
        mb="-8px"
        pl={2}
        pr={2}
        dangerouslySetInnerHTML={{ __html: file.image }}
      />
      <Box display="flex" alignItems="baseline" width="100%" pos="relative" marginTop={2}>
        <Tooltip
          openDelay={200}
          label="Preview"
        >
          <IconButton
            aria-label="play"
            variant="ghost"
            size="xs"
            disabled={isAiff}
            icon={<PlayIcon />}
            onClick={() => onPlay(file)}
          />
        </Tooltip>
        <Tooltip
          openDelay={200}
          label="Open directory"
        >
          <IconButton
            aria-label="show"
            variant="ghost"
            size="xs"
            alignSelf="center"
            icon={<ViewIcon />}
            onClick={() => onShow(file.path)}
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

export const Files = memo(({
  files,
  addFiles,
  removeFile,
  showFile,
}: IFilesProps) => {
  const { playFile } = useContext(AudioContext);

  const handleRemove = useCallback((path: string) => {
    removeFile(path);
  }, [removeFile]);

  const handleShow = useCallback((path: string) => {
    showFile(path);
  }, [showFile]);

  const handlePlay = useCallback((file: IPermutationInput) => {
    playFile(file);
  }, [playFile]);

  const onDrop = useCallback((files: any[]) => {
    const filenames: string[] = files.map(f => f.path)
    addFiles(filenames);
  }, [addFiles]);

  const onChange: React.ChangeEventHandler<HTMLInputElement> = useCallback((e) => {
    const files: string[] = [];
    for (const f of (e.target as any).files) {
      files.push(f.path);
    }
    addFiles(files);
  }, [addFiles]);

  const { getRootProps, getInputProps, isDragActive: isDrag } = useDropzone({ onDrop })

  const fileBoxes = files.map((file) => (
    <FileBox
      key={file.path}
      file={file}
      onRemove={handleRemove}
      onShow={handleShow}
      onPlay={handlePlay}
    />
  ));

  return (
    <GridItem
      rowSpan={17}
      colSpan={3}
      bg={bg}
      pt={4}
      borderRadius={20}
      display="flex"
      flexDirection="column"
      overflow="hidden"
      overflowY="scroll"
      height="100%"
      shadow="base"
    >
      <Heading textAlign="center" size="lg" color="gray.600">
        Files
      </Heading>
      <Box
        className="file-upload-container"
        borderBottom={fileBoxes.length ? '1px solid' : 'none'}
        borderBottomColor="gray.400"
      >
        <Center>
          <Button
            width="75%"
            bgColor={isDrag ? buttonBg : buttonBg}
            color="gray.50"
            _hover={{ backgroundColor: 'brand.550' }}
            fontSize="xl"
            cursor="pointer"
            shadow="base"
          >
            Select files
            <Input
              accept=".wav,.aif"
              className="file-upload"
              position="absolute"
              cursor="pointer"
              type="file"
              multiple
              onChange={onChange}
            />
            <Input
              accept=".wav,.aif"
              hidden
              type="file"
              width="0"
              height="0"
              multiple
              {...getInputProps()}
            />
          </Button>
        </Center>
      </Box>
      <Box
        height="100%"
        overflowY="scroll"
        className={isDrag ? "drag-files" : ""}
        {...getRootProps()}
        onClick={(e: React.MouseEvent) => { e.stopPropagation() }}
      >
        {fileBoxes}
      </Box>
    </GridItem>
  );
});
