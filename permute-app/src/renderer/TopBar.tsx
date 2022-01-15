import { GridItem, Heading } from "@chakra-ui/react";



export const TopBar: React.FC = () => {
  return (
  <>
  <GridItem rowSpan={1} colSpan={3} bg='green'>
    <Heading textAlign="center">Permute</Heading>
  </GridItem>
    <GridItem rowSpan={1} colSpan={9} bg='green' />
  </>
  );
}