import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import Drawer from '@mui/material/Drawer';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemText from '@mui/material/ListItemText';
import Divider from '@mui/material/Divider';
import { alpha } from '@mui/material/styles';
import MenuIcon from '@mui/icons-material/Menu';
import CloseIcon from '@mui/icons-material/Close';
import { useState } from 'react';

const navLinks = [
  { label: 'Producto', href: '#features' },
  { label: 'Precios', href: '/pricing' },
  { label: 'Descargar', href: '/downloads' },
  { label: 'Tutoriales', href: '/tutorials' },
  { label: 'Contacto', href: '/contact' },
];

export default function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <>
      <AppBar position="fixed" elevation={0}>
        <Container maxWidth="lg">
          <Toolbar disableGutters sx={{ height: 72 }}>
            {/* Logo */}
            <Typography
              variant="h6"
              component="a"
              href="/"
              sx={{
                fontWeight: 800,
                fontSize: '1.25rem',
                color: '#10b981',
                textDecoration: 'none',
                letterSpacing: '-0.02em',
                mr: 4,
              }}
            >
              Academix
            </Typography>

            {/* Desktop nav links */}
            <Stack
              direction="row"
              spacing={1}
              sx={{ display: { xs: 'none', md: 'flex' }, flex: 1 }}
            >
              {navLinks.map(({ label, href }) => (
                <Button
                  key={label}
                  href={href}
                  sx={{
                    color: 'text.secondary',
                    fontWeight: 500,
                    fontSize: '0.875rem',
                    px: 2,
                    '&:hover': {
                      color: '#f8fafc',
                      bgcolor: alpha('#94a3b8', 0.08),
                    },
                  }}
                >
                  {label}
                </Button>
              ))}
            </Stack>

            {/* Desktop CTA */}
            <Stack
              direction="row"
              spacing={1.5}
              sx={{ display: { xs: 'none', md: 'flex' } }}
            >
              <Button
                href="/auth/login"
                sx={{
                  color: 'text.secondary',
                  fontWeight: 500,
                  '&:hover': { color: '#f8fafc' },
                }}
              >
                Iniciar sesión
              </Button>
              <Button
                variant="contained"
                href="/downloads"
                size="small"
                sx={{ px: 3 }}
              >
                Comenzar gratis
              </Button>
            </Stack>

            {/* Mobile menu button */}
            <Box sx={{ display: { xs: 'flex', md: 'none' }, ml: 'auto' }}>
              <IconButton
                onClick={() => setMobileOpen(true)}
                sx={{ color: '#f8fafc' }}
              >
                <MenuIcon />
              </IconButton>
            </Box>
          </Toolbar>
        </Container>
      </AppBar>

      {/* Mobile drawer */}
      <Drawer
        anchor="right"
        open={mobileOpen}
        onClose={() => setMobileOpen(false)}
        slotProps={{
          paper: {
            sx: {
              width: 280,
              bgcolor: '#0f0f14',
              borderLeft: `1px solid ${alpha('#94a3b8', 0.08)}`,
            },
          },
        }}
      >
        <Box sx={{ p: 2, display: 'flex', justifyContent: 'flex-end' }}>
          <IconButton onClick={() => setMobileOpen(false)} sx={{ color: '#f8fafc' }}>
            <CloseIcon />
          </IconButton>
        </Box>
        <List>
          {navLinks.map(({ label, href }) => (
            <ListItem key={label} disablePadding>
              <ListItemButton
                component="a"
                href={href}
                sx={{ px: 3, '&:hover': { bgcolor: alpha('#10b981', 0.08) } }}
              >
                <ListItemText
                  primary={label}
                  slotProps={{ primary: { sx: { fontWeight: 500 } } }}
                />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
        <Divider sx={{ my: 2 }} />
        <Stack spacing={1.5} sx={{ px: 3 }}>
          <Button variant="outlined" fullWidth href="/auth/login">
            Iniciar sesión
          </Button>
          <Button variant="contained" fullWidth href="/downloads">
            Comenzar gratis
          </Button>
        </Stack>
      </Drawer>

      {/* Spacer for fixed AppBar */}
      <Toolbar sx={{ height: 72 }} />
    </>
  );
}
