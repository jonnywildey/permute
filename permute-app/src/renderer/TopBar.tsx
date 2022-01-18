import { GridItem, Heading } from "@chakra-ui/react";



export const TopBar: React.FC = () => {
  return (
  <>
  <GridItem rowSpan={1} colSpan={3} bg='green.100' borderBottom="0.5px solid" borderBottomColor="green.200" >
    <Heading textAlign="center" mt="1.5" size="lg">Permute</Heading>
  </GridItem>
    <GridItem rowSpan={1} colSpan={9} bg='green.100' borderBottom="0.5px solid" borderBottomColor="green.200" />
  </>
  );
}