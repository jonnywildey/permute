import { IconButton, Menu, MenuButton, MenuList, MenuItem, MenuGroup, useColorMode, Tooltip, useToast } from '@chakra-ui/react';
import { SunIcon, MoonIcon, CheckIcon, EditIcon, DownloadIcon, RepeatIcon } from '@chakra-ui/icons';
import { LargeHamburgerIcon } from './icons/HamburgerIcon';

interface SystemMenuProps {
  createSubfolders?: boolean;
  onCreateSubdirectoriesChange?: (createSubfolders: boolean) => void;
  onSaveScene?: () => void;
  onLoadScene?: () => void;
}

export const SystemMenu: React.FC<SystemMenuProps> = ({
  createSubfolders = false,
  onCreateSubdirectoriesChange,
  onSaveScene,
  onLoadScene
}) => {
  const { colorMode, setColorMode } = useColorMode();
  const toast = useToast();
  const fontColor = colorMode === 'dark' ? 'brand.5600' : 'gray.600';
  const bgColor = colorMode === 'dark' ? 'gray.700' : 'gray.100';
  const overwriteLabel = "Every run will overwrite existing files with the same name. If you want to keep files, you will need to move or rename them first."
  const createSubfoldersLabel = "Every run will create a new subfolder for each permutation.  This will ensure all files are kept, but may create a lot of subfolders and files."

  const handleSaveScene = () => {
    window.Electron.ipcRenderer.saveScene((filePath) => {
      if (filePath) {
        toast({
          description: 'Scene saved successfully',
          status: 'success',
          duration: 3000,
          isClosable: true,
        });
      }
    });
  };

  const handleLoadScene = () => {
    window.Electron.ipcRenderer.loadScene((filePath) => {
      if (filePath) {
        toast({
          description: 'Scene loaded successfully',
          status: 'success',
          duration: 3000,
          isClosable: true,
        });
      }
    });
  };

  return (
    <Menu closeOnSelect={false}>
      <MenuButton
        as={IconButton}
        aria-label="Options"
        icon={<LargeHamburgerIcon />}
        variant="ghost"
        color={fontColor}
        _hover={{ bg: 'brand.50' }}
        mr={2}
      />
      <MenuList
        bg={bgColor}
        borderColor="brand.150"
        backgroundColor={bgColor}
      >
        <MenuGroup title="Theme" color={fontColor} fontSize="xl">
          <MenuItem
            onClick={() => setColorMode('light')}
            _hover={{ bg: 'brand.150' }}
            color={fontColor}
            icon={<SunIcon />}
            isDisabled={colorMode === 'light'}
          >
            Light Theme
          </MenuItem>
          <MenuItem
            onClick={() => setColorMode('dark')}
            _hover={{ bg: 'brand.150' }}
            color={fontColor}
            icon={<MoonIcon />}
            isDisabled={colorMode === 'dark'}
          >
            Dark Theme
          </MenuItem>
        </MenuGroup>
        <MenuGroup title="Scene" color={fontColor} fontSize="xl">
          <MenuItem
            onClick={onSaveScene}
            _hover={{ bg: 'brand.150' }}
            color={fontColor}
            icon={<DownloadIcon />}
          >
            Save Scene
          </MenuItem>
          <MenuItem
            onClick={onLoadScene}
            _hover={{ bg: 'brand.150' }}
            color={fontColor}
            icon={<RepeatIcon />}
          >
            Load Scene
          </MenuItem>
        </MenuGroup>
        <MenuGroup title="Files" color={fontColor} fontSize="xl">
          <Tooltip openDelay={500} label={overwriteLabel}>
            <MenuItem
              onClick={() => onCreateSubdirectoriesChange?.(false)}
              _hover={{ bg: 'brand.150' }}
              color={fontColor}
              icon={!createSubfolders ? <CheckIcon /> : <EditIcon />}
              isDisabled={!createSubfolders}
            >
              Overwrite Files
            </MenuItem>
          </Tooltip>
          <Tooltip label={createSubfoldersLabel} openDelay={500}>
            <MenuItem
              onClick={() => onCreateSubdirectoriesChange?.(true)}
              _hover={{ bg: 'brand.150' }}
              color={fontColor}
              icon={createSubfolders ? <CheckIcon /> : <EditIcon />}
              isDisabled={createSubfolders}
            >
              Create Subfolders
            </MenuItem>
          </Tooltip>
        </MenuGroup>
      </MenuList>
    </Menu >
  );
}; 