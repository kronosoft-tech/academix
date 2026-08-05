import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Rating from '@mui/material/Rating';
import { alpha } from '@mui/material/styles';
import FormatQuoteIcon from '@mui/icons-material/FormatQuote';

const testimonials = [
  {
    name: 'María López',
    role: 'Directora',
    org: 'Academia Bilingüe Luz',
    avatar: 'ML',
    text: 'Usar Academix es como tener un asistente administrativo que nunca descansa. Redujimos el tiempo de gestión en un 60% y ahora el equipo puede concentrarse en lo importante: los estudiantes.',
    rating: 5,
  },
  {
    name: 'Carlos Méndez',
    role: 'Administrador',
    org: 'Centro TechPro',
    avatar: 'CM',
    text: 'Los reportes mensuales que antes nos tomaban un día entero ahora se generan en segundos. El control de pagos automatizado cambió por completo nuestra operación financiera.',
    rating: 5,
  },
  {
    name: 'Ana Rodríguez',
    role: 'Gerente Académica',
    org: 'Instituto Musical Armonía',
    avatar: 'AR',
    text: 'Lo que más valoro es la simplicidad. Cada profesor tiene acceso a lo que necesita sin perderse en menús complicados. La curva de aprendizaje fue prácticamente cero.',
    rating: 5,
  },
];

export default function TestimonialsSection() {
  return (
    <Box
      component="section"
      sx={{
        py: { xs: 10, md: 16 },
        bgcolor: alpha('#0f0f14', 0.5),
      }}
    >
      <Container maxWidth="lg">
        <Stack sx={{ alignItems: 'center', gap: 2, mb: { xs: 6, md: 10 } }}>
          <Typography variant="h2" sx={{ textAlign: 'center' }}>
            Equipos que confían en Academix.
          </Typography>
          <Typography
            variant="body1"
            sx={{ textAlign: 'center', maxWidth: 600, color: 'text.secondary' }}
          >
            Más de 500 academias usan Academix a diario para transformar
            su administración educativa.
          </Typography>
        </Stack>

        <Grid container spacing={3}>
          {testimonials.map(({ name, role, org, avatar, text, rating }) => (
            <Grid key={name} size={{ xs: 12, md: 4 }}>
              <Card
                sx={{
                  p: 4,
                  height: '100%',
                  borderRadius: 3,
                  display: 'flex',
                  flexDirection: 'column',
                }}
              >
                <CardContent sx={{ p: 0, flex: 1, display: 'flex', flexDirection: 'column' }}>
                  <FormatQuoteIcon
                    sx={{ fontSize: 32, color: alpha('#10b981', 0.4), mb: 2 }}
                  />
                  <Typography
                    variant="body1"
                    sx={{
                      flex: 1,
                      color: alpha('#f8fafc', 0.9),
                      fontSize: '0.95rem',
                      lineHeight: 1.8,
                      mb: 3,
                    }}
                  >
                    {text}
                  </Typography>
                  <Rating
                    value={rating}
                    readOnly
                    size="small"
                    sx={{
                      mb: 2,
                      '& .MuiRating-iconFilled': { color: '#10b981' },
                    }}
                  />
                  <Stack sx={{ flexDirection: 'row', gap: 2, alignItems: 'center' }}>
                    <Avatar
                      sx={{
                        bgcolor: alpha('#10b981', 0.15),
                        color: '#10b981',
                        fontWeight: 600,
                        width: 40,
                        height: 40,
                      }}
                    >
                      {avatar}
                    </Avatar>
                    <Box>
                      <Typography variant="body2" sx={{ fontWeight: 600, color: '#f8fafc' }}>
                        {name}
                      </Typography>
                      <Typography variant="body2" sx={{ color: 'text.secondary', fontSize: '0.8rem' }}>
                        {role}, {org}
                      </Typography>
                    </Box>
                  </Stack>
                </CardContent>
              </Card>
            </Grid>
          ))}
        </Grid>
      </Container>
    </Box>
  );
}
