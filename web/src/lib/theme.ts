import { createTheme, alpha } from '@mui/material/styles';

const emerald = {
  50: '#ecfdf5',
  100: '#d1fae5',
  200: '#a7f3d0',
  300: '#6ee7b7',
  400: '#34d399',
  500: '#10b981',
  600: '#059669',
  700: '#047857',
  800: '#065f46',
  900: '#064e3b',
};

export const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: emerald[500],
      light: emerald[400],
      dark: emerald[700],
      contrastText: '#ffffff',
    },
    secondary: {
      main: '#8b5cf6',
      light: '#a78bfa',
      dark: '#7c3aed',
    },
    background: {
      default: '#09090b',
      paper: '#0f0f14',
    },
    text: {
      primary: '#f8fafc',
      secondary: '#94a3b8',
    },
    divider: alpha('#94a3b8', 0.12),
  },
  typography: {
    fontFamily: '"Inter", "SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    h1: {
      fontSize: 'clamp(2.5rem, 5vw, 4.5rem)',
      fontWeight: 700,
      lineHeight: 1.1,
      letterSpacing: '-0.02em',
    },
    h2: {
      fontSize: 'clamp(2rem, 4vw, 3rem)',
      fontWeight: 700,
      lineHeight: 1.2,
      letterSpacing: '-0.01em',
    },
    h3: {
      fontSize: 'clamp(1.5rem, 3vw, 2rem)',
      fontWeight: 600,
      lineHeight: 1.3,
    },
    h4: {
      fontSize: '1.25rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h5: {
      fontSize: '1.125rem',
      fontWeight: 600,
      lineHeight: 1.4,
    },
    h6: {
      fontSize: '1rem',
      fontWeight: 600,
      lineHeight: 1.5,
    },
    body1: {
      fontSize: '1.125rem',
      lineHeight: 1.7,
      color: '#94a3b8',
    },
    body2: {
      fontSize: '0.875rem',
      lineHeight: 1.6,
      color: '#64748b',
    },
    button: {
      textTransform: 'none',
      fontWeight: 600,
    },
  },
  shape: {
    borderRadius: 16,
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          backgroundColor: '#09090b',
          scrollBehavior: 'smooth',
        },
        '::selection': {
          backgroundColor: alpha(emerald[500], 0.3),
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 12,
          padding: '12px 28px',
          fontSize: '1rem',
          fontWeight: 600,
          boxShadow: 'none',
          '&:hover': {
            boxShadow: 'none',
          },
          '&.MuiButton-containedPrimary': {
            background: `linear-gradient(135deg, ${emerald[500]} 0%, ${emerald[600]} 100%)`,
            '&:hover': {
              background: `linear-gradient(135deg, ${emerald[400]} 0%, ${emerald[500]} 100%)`,
              boxShadow: `0 8px 32px ${alpha(emerald[500], 0.3)}`,
            },
          },
          '&.MuiButton-outlinedPrimary': {
            borderColor: alpha(emerald[500], 0.5),
            '&:hover': {
              borderColor: emerald[400],
              backgroundColor: alpha(emerald[500], 0.08),
            },
          },
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
          backgroundColor: alpha('#1e1e2e', 0.6),
          backdropFilter: 'blur(20px)',
          border: `1px solid ${alpha('#94a3b8', 0.08)}`,
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
          '&:hover': {
            border: `1px solid ${alpha(emerald[500], 0.3)}`,
            boxShadow: `0 8px 32px ${alpha(emerald[500], 0.08)}`,
            transform: 'translateY(-2px)',
          },
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
        },
      },
    },
    MuiAppBar: {
      styleOverrides: {
        root: {
          backgroundImage: 'none',
          backgroundColor: alpha('#09090b', 0.8),
          backdropFilter: 'blur(20px)',
          borderBottom: `1px solid ${alpha('#94a3b8', 0.08)}`,
          boxShadow: 'none',
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          fontWeight: 500,
        },
        outlined: {
          borderColor: alpha('#94a3b8', 0.2),
        },
      },
    },
    MuiDivider: {
      styleOverrides: {
        root: {
          borderColor: alpha('#94a3b8', 0.08),
        },
      },
    },
  },
});
