import { Box, GridItem, Heading, Image } from "@chakra-ui/react";
import logo from '../img/logo.png';


const bg = 'brand.150';

export interface TopBarProps {
  openWelcome: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({ openWelcome }) => {
  return (
    <>
      <GridItem
        rowSpan={1} colSpan={12} bg={bg} 
        height="100%"
        borderRadius={20}
        shadow="lg"

      >
        <Box onClick={openWelcome} cursor="pointer" display="flex"
        alignItems="center" height="100%">
        <Image src={logo} width={45} height={45} padding={1} ml={2} />
        <Heading ml={3} textAlign="center" size="lg" color="gray.800" display="inline">Permute</Heading>
        </Box>
      </GridItem>
    </>
  );
}