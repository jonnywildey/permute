import { IconButton, Menu, MenuButton, MenuList, MenuItem, MenuGroup, useColorMode } from '@chakra-ui/react';
import { SunIcon, MoonIcon } from '@chakra-ui/icons';
import { LargeHamburgerIcon } from './icons/HamburgerIcon';

export const SystemMenu: React.FC = () => {
  const { colorMode, setColorMode } = useColorMode();
  const fontColor = colorMode === 'dark' ? 'brand.5600' : 'gray.600';
  const bgColor = colorMode === 'dark' ? 'gray.700' : 'gray.100';
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
        <MenuGroup title="Theme" color={fontColor}>
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
      </MenuList>
    </Menu>
  );
}; 