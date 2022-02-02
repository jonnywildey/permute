import { Box, GridItem, Heading, Input, IconButton, CloseButton, PropsOf, Text, Button, Center } from "@chakra-ui/react";
import { ViewIcon } from "@chakra-ui/icons"
import { useContext, useState } from "react";
import { AudioContext } from "./AudioContext";
import { PlayIcon } from "./PlayIcon";
import type { IPermutationInput } from "permute-node";

export interface IFilesProps {
  files: IPermutationInput[];
  addFiles: (files: string[]) => void;
  removeFile: (file: string) => void;
  showFile: (file: string) => void;
}

const buttonBg = "brand.500";
const bg = "brand.25";
const fileBorderColour = "brand.150";

export const Files: React.FC<IFilesProps> = ({ files, addFiles, removeFile, showFile }) => {
  const [isDrag, setDrag] = useState(false);
  const { playFile } = useContext(AudioContext);

  const onDrop: React.DragEventHandler<HTMLInputElement> = (e) => {
    let files: string[] = [];
    for (const f of (e.dataTransfer as any).files) {
      files.push(f.path);
    };
    addFiles(files);
    setDrag(false);
  };
  const onChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
    let files: string[] = [];
    for (const f of (e.target as any).files) {
      files.push(f.path);
    };
    addFiles(files);
  }

  const fileBoxes = files.map((file, i) => {
    const props: PropsOf<typeof Box> = {
      key: file,
      borderBottom: "1px solid",
      borderBottomColor: fileBorderColour,
      color: "gray.700"
    };
    if (i === 0) {
      props.borderTop = "1px solid";
      props.borderTopColor = fileBorderColour;
    }
    return (<Box {...props}>
      <Heading
        size="sm"
        width="80%"
        display="inline"
        color="gray.600"
        pl={2}
      >{file.name}</Heading>
      <CloseButton
        display="inline"
        float="right"
        size="sm"
        onClick={() => removeFile(file.path)}
      />
      <Box
        display="flex"
        alignItems="center"
        width="100%"
      >

        <IconButton
          aria-label="show"
          variant="ghost"
          size="sm"
          icon={<PlayIcon />}
          onClick={() => playFile(file.path)}
        />

        <IconButton
          aria-label="show"
          variant="ghost"
          size="sm"
          icon={<ViewIcon />}
          onClick={() => showFile(file.path)}
        />
      </Box>
    </Box>);
  });

  return <GridItem rowSpan={17} colSpan={3} bg={bg} pt={4}>
    <Heading textAlign="center" size="lg" color="gray.600">Files</Heading>
    <Box className="file-upload-container"
    >
      <Center>
        <Button width="75%" bgColor={isDrag ? buttonBg : buttonBg} color="gray.800">Select files
          <Input
            accept=".wav"
            className="file-upload"
            position="absolute"
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
    <Box overflowY="scroll" height="370px">
      {fileBoxes}
    </Box>
  </GridItem>

}