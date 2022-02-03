import { extendTheme } from '@chakra-ui/react';
import { createBreakpoints } from '@chakra-ui/theme-tools';

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
      brand: {
        25: "#bed3ee52",
        50: "#CDBCD3",
        75: "#3ee9c414",
        100: "#a293d6c7",
        125: "#93c8d6c7",
        150: "#bed3efb5",
        200: "#3A3659",
        300: "#52B74D",
        400: "#6D7D99",
        500: "#CBA7DA",
        600: "#3EE8C5",
        650: "#3ee9c44d",
        700: "#3ee8c5d9"
      }
    },
    fonts: {
      heading: 'dongle',
      body: 'dongle',
    },
    components: {
      Toast: {
        defaultProps: {
          colorScheme: 'purple',
        },
      },
    }
  }
);
