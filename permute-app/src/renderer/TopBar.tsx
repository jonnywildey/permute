import { GridItem, Heading, Image } from "@chakra-ui/react";
import logo from '../../assets/logo.png';


const bg = 'brand.150';
const borderColour = "gray.200"

export const TopBar: React.FC = () => {
  return (
    <>
      <GridItem
        rowSpan={2} colSpan={3} bg={bg} borderBottom="0.5px solid"
        borderBottomColor={borderColour} display="flex"
        alignItems="center"
      >
        <Image src={logo} width={45} height={45} padding={1} ml={2} />
        <Heading ml={3} textAlign="center" mt="1.5" size="lg" color="gray.800">Permute</Heading>
      </GridItem>
      <GridItem rowSpan={2} colSpan={9} bg={bg} borderBottom="0.5px solid" borderBottomColor={borderColour} />
    </>
  );
}