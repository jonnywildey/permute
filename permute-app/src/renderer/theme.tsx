import { extendTheme } from '@chakra-ui/react';
import { createBreakpoints } from '@chakra-ui/theme-tools';
import '@fontsource/dongle/400.css';
import '@fontsource/dongle/300.css';
import '@fontsource/dongle/700.css';

export const darkTheme = extendTheme(
  {
    colors: {
      brand: {

        // 25: '#36404c52',
        25: '#3647668c',
        // 25: '#2f435ac7',



        50: '#CDBCD3',
        75: '#3ee9c414',
        100: '#a293d6c7',
        125: '#93c8d6c7',
        126: '#93c8d637',
        // 150: '#bed3ef',
        150: '#31547e',
        160: '#242e3c6e',
        175: '#4A8BA3',
        200: '#49bd90',
        // 200: '#3A3659',
        210: '#39ad80BB',
        300: '#A66FB8',
        400: '#6D7D99',
        500: '#CBA7DA',
        550: '#A66FB8',
        600: '#0fd7e3',
        650: '#3ee9c44d',
        675: '#b2eff9e3',
        700: '#3ee8c5d9',

        5600: "#e5fffe",
        5700: "#e2e2ed",
        5800: "#eef9f4"
      },
    },
    fonts: {
      heading: 'Dongle, sans-serif',
      body: 'Dongle, sans-serif',
    },
    components: {
      Toast: {
        defaultProps: {
          colorScheme: 'purple',
        },
      },
    },
  },
  createBreakpoints({
    sm: '1200em',
    md: '1200em',
    lg: '1200em',
    xl: '1200em',
    '2xl': '1200em',
  })
);

export const lightTheme = extendTheme(
  {
    colors: {
      brand: {
        25: '#bed3ee52',
        50: '#CDBCD3',
        75: '#3ee9c414',
        100: '#a293d6c7',
        125: '#93c8d6c7',
        150: '#bed3efb5',
        175: '#4A8BA3',
        200: '#3A3659',
        210: '#3A3659BB',
        300: '#52B74D',
        400: '#6D7D99',
        500: '#CBA7DA',
        550: '#A66FB8',
        600: '#3EE8C5',
        650: '#3ee9c44d',
        675: '#3ee9c4b0',
        700: '#3ee8c5d9',
      },
    },
    fonts: {
      heading: 'Dongle, sans-serif',
      body: 'Dongle, sans-serif',
    },
    components: {
      Toast: {
        defaultProps: {
          colorScheme: 'purple',
        },
      },
    },
  },
  createBreakpoints({
    sm: '1200em',
    md: '1200em',
    lg: '1200em',
    xl: '1200em',
    '2xl': '1200em',
  })
);
