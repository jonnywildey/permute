import { extendTheme, withDefaultSize } from '@chakra-ui/react';
import { createBreakpoints } from '@chakra-ui/theme-tools';

import '@fontsource/dongle/400.css'
import '@fontsource/dongle/300.css'
import '@fontsource/dongle/700.css'

export const theme = extendTheme(

  createBreakpoints({
    sm: '1200em',
    md: '1200em',
    lg: '1200em',
    xl: '1200em',
    '2xl': '1200em',
  }),
  {
    colors: {
      primary: "#EC598B",
      secondary: "#76FFC0"
    },
    fonts: {
    heading: 'dongle',
    body: 'dongle',
  },
  }
);
