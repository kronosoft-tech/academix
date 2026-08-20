import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Link from '@mui/material/Link';
import Divider from '@mui/material/Divider';
import IconButton from '@mui/material/IconButton';
import { alpha } from '@mui/material/styles';
import XIcon from '@mui/icons-material/X';
import LinkedInIcon from '@mui/icons-material/LinkedIn';
import YouTubeIcon from '@mui/icons-material/YouTube';
import GitHubIcon from '@mui/icons-material/GitHub';

const footerLinks = [
  {
    title: 'Producto',
    links: [
      { label: 'Funcionalidades', href: '#features' },
      { label: 'Precios', href: '/pricing' },
      { label: 'Descargar', href: '/downloads' },
      { label: 'Actualizaciones', href: '/downloads' },
    ],
  },
  {
    title: 'Recursos',
    links: [
      { label: 'Tutoriales', href: '/tutorials' },
      { label: 'FAQ', href: '/faq' },
      { label: 'Documentación', href: '/tutorials' },
      { label: 'Comunidad', href: '/contact' },
    ],
  },
  {
    title: 'Empresa',
    links: [
      { label: 'Contacto', href: '/contact' },
      { label: 'Blog', href: '/tutorials' },
      { label: 'Privacidad', href: '/privacidad' },
      { label: 'Términos', href: '/terminos' },
    ],
  },
];

export default function Footer() {
  return (
    <Box
      component="footer"
      sx={{
        borderTop: `1px solid ${alpha('#94a3b8', 0.08)}`,
        pt: { xs: 6, md: 10 },
        pb: { xs: 4, md: 6 },
        bgcolor: alpha('#0f0f14', 0.5),
      }}
    >
      <Container maxWidth="lg">
        <Grid container spacing={4}>
          {/* Brand column */}
          <Grid size={{ xs: 12, md: 4 }}>
            <Typography
              variant="h6"
              sx={{ fontWeight: 800, color: '#10b981', mb: 2 }}
            >
              Academix
            </Typography>
            <Typography
              variant="body2"
              sx={{ color: 'text.secondary', maxWidth: 280, mb: 3, lineHeight: 1.7 }}
            >
              La plataforma de gestión académica que simplifica la administración
              para que tu equipo se concentre en lo que importa: enseñar.
            </Typography>
            <Stack direction="row" spacing={1}>
              <IconButton
                size="small"
                sx={{ color: 'text.secondary', '&:hover': { color: '#10b981' } }}
              >
                <XIcon fontSize="small" />
              </IconButton>
              <IconButton
                size="small"
                sx={{ color: 'text.secondary', '&:hover': { color: '#10b981' } }}
              >
                <LinkedInIcon fontSize="small" />
              </IconButton>
              <IconButton
                size="small"
                sx={{ color: 'text.secondary', '&:hover': { color: '#10b981' } }}
              >
                <YouTubeIcon fontSize="small" />
              </IconButton>
              <IconButton
                size="small"
                sx={{ color: 'text.secondary', '&:hover': { color: '#10b981' } }}
              >
                <GitHubIcon fontSize="small" />
              </IconButton>
            </Stack>
          </Grid>

          {/* Link columns */}
          {footerLinks.map(({ title, links }) => (
            <Grid key={title} size={{ xs: 6, sm: 4, md: 2.66 }}>
              <Typography
                variant="body2"
                sx={{ fontWeight: 600, color: '#f8fafc', mb: 2 }}
              >
                {title}
              </Typography>
              <Stack spacing={1.5}>
                {links.map(({ label, href }) => (
                  <Link
                    key={label}
                    href={href}
                    underline="none"
                    sx={{
                      color: 'text.secondary',
                      fontSize: '0.875rem',
                      transition: 'color 0.2s',
                      '&:hover': { color: '#f8fafc' },
                    }}
                  >
                    {label}
                  </Link>
                ))}
              </Stack>
            </Grid>
          ))}
        </Grid>

        <Divider sx={{ my: { xs: 4, md: 6 } }} />

        <Stack
          sx={{
            flexDirection: { xs: 'column', sm: 'row' },
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 2,
          }}
        >
          <Typography variant="body2" sx={{ color: alpha('#94a3b8', 0.6), fontSize: '0.8rem' }}>
            &copy; {new Date().getFullYear()} Academix. Todos los derechos reservados.
          </Typography>
          <Stack sx={{ flexDirection: 'row', gap: 3 }}>
            <Link
              href="/privacidad"
              underline="none"
              sx={{ color: alpha('#94a3b8', 0.6), fontSize: '0.8rem', '&:hover': { color: '#f8fafc' } }}
            >
              Privacidad
            </Link>
            <Link
              href="/terminos"
              underline="none"
              sx={{ color: alpha('#94a3b8', 0.6), fontSize: '0.8rem', '&:hover': { color: '#f8fafc' } }}
            >
              Términos
            </Link>
            <Link
              href="/cookies"
              underline="none"
              sx={{ color: alpha('#94a3b8', 0.6), fontSize: '0.8rem', '&:hover': { color: '#f8fafc' } }}
            >
              Cookies
            </Link>
          </Stack>
        </Stack>
      </Container>
    </Box>
  );
}
