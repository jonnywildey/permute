import { Box, GridItem, Heading, Input } from "@chakra-ui/react";

export interface IFilesProps {
  files: string[];
  refreshState: () => void;
}

export const Files: React.FC<IFilesProps> = ({ files, refreshState }) => {
  const onDrop: React.DragEventHandler<HTMLInputElement> = (e) => {
      for (const f of (e.dataTransfer as any).files) {
        console.log('File(s) you dragged here: ', f.path);
        window.Electron.ipcRenderer.addFile(f.path);
      };
      refreshState();
    };
  const onChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
      for (const f of (e.target as any).files) {
        console.log('File(s) you uploaded here: ', f.path)
        window.Electron.ipcRenderer.addFile(f.path);
      };
      refreshState();
  }

  const fileBoxes = files.map(file => (
    <Box key={file}>{file}</Box>
  ))
  
  return <GridItem rowSpan={9} colSpan={3} bg='yellow.50'>
    <Heading textAlign="center" size="lg">Files</Heading>

    <Input type="file" multiple onDrop={onDrop} onChange={onChange} />

    {fileBoxes}
    
  </GridItem>

}