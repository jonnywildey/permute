import { Box, GridItem, Heading, Input, IconButton, CloseButton, PropsOf, Text, Button, Center } from "@chakra-ui/react";
import { IFileStat } from "main/IFileStat";
import { ViewIcon } from  "@chakra-ui/icons"
import { useState } from "react";

export interface IFilesProps {
  files: IFileStat[];
  addFiles: (files: string[]) => void;
  removeFile: (file: string) => void;
  showFile: (file: string) => void;
}

export const Files: React.FC<IFilesProps> = ({ files, addFiles, removeFile, showFile }) => {
  const [isDrag, setDrag] = useState(false);

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
      key: file.name,
      borderBottom: "1px solid",
      borderBottomColor: "yellow.200",
      color: "gray.700"
    };
    if (i === 0) {
      props.borderTop = "1px solid";
      props.borderTopColor = "yellow.200";
    }
    return (<Box {...props}>
      <Heading 
        size="sm" 
        width="80%"         
        display="inline"
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
      justifyContent="space-between"
      alignItems="center"
      >
        <IconButton  
          aria-label="show" 
          variant="ghost"
          size="sm"
          icon={<ViewIcon  />} 
          onClick={() => showFile(file.path)} 
          />
        <Text
          display="inline"
          float="right"
          pr={2}
        >{file.sizeMb} mb</Text>
      </Box>
    </Box>);
  });

  return <GridItem rowSpan={17} colSpan={3} bg='yellow.50' pt={4}>
    <Heading textAlign="center" size="lg">Files</Heading>
    <Box className="file-upload-container"
    >
      <Center>
        <Button width="75%" bgColor={isDrag ? "pink.200" : "pink.300"}>Select files 
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
    <Box overflowY="scroll" height="386px">
    {fileBoxes}
    </Box>
  </GridItem>

}