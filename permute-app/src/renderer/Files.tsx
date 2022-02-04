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
  Image,
  Text,
} from '@chakra-ui/react';
import { ViewIcon } from '@chakra-ui/icons';
import { useContext, useState } from 'react';
import type { IPermutationInput } from 'permute-node';
import { AudioContext } from './AudioContext';
import { PlayIcon } from './PlayIcon';
import { displayTime } from './displayTime';

export interface IFilesProps {
  files: IPermutationInput[];
  addFiles: (files: string[]) => void;
  removeFile: (file: string) => void;
  showFile: (file: string) => void;
}

const buttonBg = 'brand.500';
const bg = 'brand.25';
const fileBorderColour = 'brand.150';

export const Files: React.FC<IFilesProps> = ({
  files,
  addFiles,
  removeFile,
  showFile,
}) => {
  const [isDrag, setDrag] = useState(false);
  const { playFile } = useContext(AudioContext);

  const onDrop: React.DragEventHandler<HTMLInputElement> = (e) => {
    const files: string[] = [];
    for (const f of (e.dataTransfer as any).files) {
      files.push(f.path);
    }
    addFiles(files);
    setDrag(false);
  };
  const onChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
    const files: string[] = [];
    for (const f of (e.target as any).files) {
      files.push(f.path);
    }
    addFiles(files);
  };

  const fileBoxes = files.map((file, i) => {
    const props: PropsOf<typeof Box> = {
      key: file.path,
      borderBottom: '1px solid',
      borderBottomColor: fileBorderColour,
      pos: 'relative',
      color: 'gray.700',
    };
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
          <CloseButton
            display="inline"
            float="right"
            color="gray.600"
            size="sm"
            onClick={() => removeFile(file.path)}
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
        <Box display="flex" alignItems="baseline" width="100%" pos="relative">
          <IconButton
            aria-label="show"
            variant="ghost"
            size="xs"
            icon={<PlayIcon />}
            onClick={() => playFile(file)}
          />
          <IconButton
            aria-label="show"
            variant="ghost"
            size="xs"
            alignSelf="center"
            icon={<ViewIcon />}
            onClick={() => showFile(file.path)}
          />
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
            fontSize="lg"
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
              onDrop={onDrop}
              onChange={onChange}
              onDragEnter={() => setDrag(true)}
              onDragLeave={() => setDrag(false)}
            />
          </Button>
        </Center>
      </Box>
      <Box height="100%" overflowY="scroll">
        {fileBoxes}
      </Box>
    </GridItem>
  );
};
