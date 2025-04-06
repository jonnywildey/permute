import { Box, GridItem, Heading, Image as ChakraImage } from '@chakra-ui/react';
import logo from '../img/logo.png';
import logoDark from '../img/logo-dark.png';
import { useEffect } from 'react';
import { useColorMode } from '@chakra-ui/react';
import { SystemMenu } from './SystemMenu';

interface TopBarProps {
  openWelcome: () => void;
  overwriteFiles?: boolean;
  createSubdirectories?: boolean;
  onOverwriteChange?: (overwrite: boolean) => void;
  onCreateSubdirectoriesChange?: (createSubfolders: boolean) => void;
  onSaveScene?: () => void;
  onLoadScene?: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  openWelcome,
  createSubdirectories: createSubfolders,
  onCreateSubdirectoriesChange,
  onSaveScene,
  onLoadScene
}) => {
  const { colorMode } = useColorMode();

  useEffect(() => {
    // Preload both images
    const lightImg = new window.Image();
    lightImg.src = logo;
    const darkImg = new window.Image();
    darkImg.src = logoDark;
  }, []);

  return (
    <>
      <GridItem
        rowSpan={1}
        colSpan={12}
        bg={colorMode === 'dark' ? 'brand.160' : 'brand.150'}
        height="100%"
        borderRadius={20}
        shadow="lg"
      >
        <Box
          display="flex"
          alignItems="center"
          height="100%"
          justifyContent="space-between"
        >
          <Box
            onClick={openWelcome}
            cursor="pointer"
            display="flex"
            alignItems="center"
          >
            <ChakraImage
              src={colorMode === 'dark' ? logoDark : logo}
              width={45}
              height={45}
              padding={1}
              ml={2}
              loading="eager"
              crossOrigin="anonymous"
            />
            <Heading
              ml={3}
              textAlign="center"
              size="lg"
              color="brand.5800"
              display="inline"
            >
              Permute
            </Heading>
          </Box>
          <SystemMenu
            createSubfolders={createSubfolders}
            onCreateSubdirectoriesChange={onCreateSubdirectoriesChange}
            onSaveScene={onSaveScene}
            onLoadScene={onLoadScene}
          />
        </Box>
      </GridItem>
    </>
  );
};
