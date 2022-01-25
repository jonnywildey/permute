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
        25: "#bed3ee38",
        50: "#CDBCD3",
        75: "#3ee9c414",
        100: "#A293D6",
        150: "#BED3EF",
        200: "#3A3659",
        300: "#52B74D",
        400: "#6D7D99",
        500: "#CBA7DA",
        600: "#3EE8C5",
        650: "#3ee9c44d",
        700: "#C0BDF0"
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
