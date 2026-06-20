import { extendTheme } from '@chakra-ui/react';
import { createBreakpoints } from '@chakra-ui/theme-tools';
import '@fontsource/dongle/400.css';
import '@fontsource/dongle/300.css';
import '@fontsource/dongle/700.css';

const colors = {
  brand: {
    25: { light: '#bed3ee52', dark: '#3647668c' },
    50: { light: '#CDBCD3', dark: '#CDBCD3' },
    75: { light: '#3ee9c414', dark: '#3ee9c414' },
    100: { light: '#a293d6c7', dark: '#a293d6c7' },
    125: { light: '#93c8d6c7', dark: '#93c8d6c7' },
    126: { light: '#93c8d637', dark: '#93c8d637' },
    150: { light: '#bed3efb5', dark: '#31547e' },
    160: { light: '#e8edf552', dark: '#242e3c6e' },
    175: { light: '#4A8BA3', dark: '#4A8BA3' },
    200: { light: '#3A3659', dark: '#49bd90' },
    210: { light: '#3A3659BB', dark: '#39ad80BB' },
    300: { light: '#52B74D', dark: '#A66FB8' },
    400: { light: '#6D7D99', dark: '#6D7D99' },
    500: { light: '#CBA7DA', dark: '#CBA7DA' },
    525: { light: '#a293d6', dark: '#CBA7DA' },
    550: { light: '#A66FB8', dark: '#A66FB8' },
    600: { light: '#3EE8C5', dark: '#0fd7e3' },
    650: { light: '#3ee9c44d', dark: '#3ee9c44d' },
    675: { light: '#3ee9c4b0', dark: '#b2eff9e3' },
    700: { light: '#3ee8c5d9', dark: '#3ee8c5d9' },
    5600: { light: "#2a4365", dark: "#e5fffe" },
    5700: { light: "#1a365d", dark: "#e2e2ed" },
    5800: { light: "#2c5282", dark: "#eef9f4" }
  },
};

const semanticTokens = {
  colors: {
    'brand.25': {
      default: colors.brand[25].light,
      _dark: colors.brand[25].dark,
    },
    'brand.50': {
      default: colors.brand[50].light,
      _dark: colors.brand[50].dark,
    },
    'brand.75': {
      default: colors.brand[75].light,
      _dark: colors.brand[75].dark,
    },
    'brand.100': {
      default: colors.brand[100].light,
      _dark: colors.brand[100].dark,
    },
    'brand.125': {
      default: colors.brand[125].light,
      _dark: colors.brand[125].dark,
    },
    'brand.126': {
      default: colors.brand[126].light,
      _dark: colors.brand[126].dark,
    },
    'brand.150': {
      default: colors.brand[150].light,
      _dark: colors.brand[150].dark,
    },
    'brand.160': {
      default: colors.brand[160].light,
      _dark: colors.brand[160].dark,
    },
    'brand.175': {
      default: colors.brand[175].light,
      _dark: colors.brand[175].dark,
    },
    'brand.200': {
      default: colors.brand[200].light,
      _dark: colors.brand[200].dark,
    },
    'brand.210': {
      default: colors.brand[210].light,
      _dark: colors.brand[210].dark,
    },
    'brand.300': {
      default: colors.brand[300].light,
      _dark: colors.brand[300].dark,
    },
    'brand.400': {
      default: colors.brand[400].light,
      _dark: colors.brand[400].dark,
    },
    'brand.500': {
      default: colors.brand[500].light,
      _dark: colors.brand[500].dark,
    },
    'brand.525': {
      default: colors.brand[525].light,
      _dark: colors.brand[525].dark,
    },
    'brand.550': {
      default: colors.brand[550].light,
      _dark: colors.brand[550].dark,
    },
    'brand.600': {
      default: colors.brand[600].light,
      _dark: colors.brand[600].dark,
    },
    'brand.650': {
      default: colors.brand[650].light,
      _dark: colors.brand[650].dark,
    },
    'brand.675': {
      default: colors.brand[675].light,
      _dark: colors.brand[675].dark,
    },
    'brand.700': {
      default: colors.brand[700].light,
      _dark: colors.brand[700].dark,
    },
    'brand.5600': {
      default: colors.brand[5600].light,
      _dark: colors.brand[5600].dark,
    },
    'brand.5700': {
      default: colors.brand[5700].light,
      _dark: colors.brand[5700].dark,
    },
    'brand.5800': {
      default: colors.brand[5800].light,
      _dark: colors.brand[5800].dark,
    },
  },
};

export const theme = extendTheme(
  {
    config: {
      initialColorMode: 'system',
      useSystemColorMode: true,
    },
    semanticTokens,
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

