import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import { alpha } from '@mui/material/styles';
import RocketLaunchIcon from '@mui/icons-material/RocketLaunch';

export default function CTASection() {
  return (
    <Box
      component="section"
      sx={{
        py: { xs: 10, md: 16 },
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Background glow */}
      <Box
        sx={{
          position: 'absolute',
          inset: 0,
          background: `radial-gradient(ellipse 80% 50% at 50% 50%, ${alpha('#10b981', 0.1)} 0%, transparent 70%)`,
        }}
      />

      <Container maxWidth="md" sx={{ position: 'relative', zIndex: 1 }}>
        <Stack sx={{ alignItems: 'center', gap: 4, textAlign: 'center' }}>
          <Box
            sx={{
              width: 64,
              height: 64,
              borderRadius: 3,
              bgcolor: alpha('#10b981', 0.1),
              border: `1px solid ${alpha('#10b981', 0.2)}`,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <RocketLaunchIcon sx={{ fontSize: 32, color: '#10b981' }} />
          </Box>

          <Typography variant="h2">
            Comienza hoy. Es gratis.
          </Typography>

          <Typography
            variant="body1"
            sx={{ maxWidth: 500, color: 'text.secondary' }}
          >
            Descarga Academix y transforma la forma en que gestionas
            tu academia. Sin tarjeta de crédito, sin complicaciones.
          </Typography>

          <Stack sx={{ flexDirection: { xs: 'column', sm: 'row' }, gap: 2 }}>
            <Button
              variant="contained"
              size="large"
              href="/downloads"
              sx={{ px: 5, py: 1.5 }}
            >
              Descargar ahora
            </Button>
            <Button
              variant="outlined"
              size="large"
              href="/contact"
              sx={{ px: 5, py: 1.5 }}
            >
              Solicitar demo
            </Button>
          </Stack>
        </Stack>
      </Container>
    </Box>
  );
}
