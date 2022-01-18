import { extendTheme } from '@chakra-ui/react';
import { createBreakpoints } from '@chakra-ui/theme-tools';

export const theme = extendTheme(

  createBreakpoints({
    sm: '1200em',
    md: '1200em',
    lg: '1200em',
    xl: '1200em',
    '2xl': '1200em',
  })
);
