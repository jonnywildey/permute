import { GridItem, Heading, Input } from "@chakra-ui/react";



export const Files: React.FC = () => {
  const onDrop: React.DragEventHandler<HTMLInputElement> = (e) => {
      for (const f of (e.dataTransfer as any).files) {
        console.log('File(s) you dragged here: ', f.path)
      };
    };
  const onChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
      for (const f of (e.target as any).files) {
        console.log('File(s) you uploaded here: ', f.path)
      };
  }
  
  return <GridItem rowSpan={9} colSpan={3} bg='tomato'>
    <Heading textAlign="center" size="lg">Files</Heading>

    <Input type="file" multiple onDrop={onDrop} onChange={onChange} />
    
  </GridItem>

}