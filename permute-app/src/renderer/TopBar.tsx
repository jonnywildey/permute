import { GridItem, Heading, Image } from "@chakra-ui/react";
import logo from '../../assets/logo.png';



export const TopBar: React.FC = () => {
  return (
  <>
  <GridItem 
  rowSpan={1} colSpan={3} bg='green.100' borderBottom="0.5px solid" 
  borderBottomColor="green.200" display="flex" 
  alignItems="center"
  >
    <Image src={logo} width={45} height={45} padding={1} ml={2} />
    <Heading ml={3} textAlign="center" mt="1.5" size="lg">Permute</Heading>
  </GridItem>
    <GridItem rowSpan={1} colSpan={9} bg='green.100' borderBottom="0.5px solid" borderBottomColor="green.200" />
  </>
  );
}