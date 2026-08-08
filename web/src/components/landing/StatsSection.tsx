import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Grid from '@mui/material/Grid';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import { alpha } from '@mui/material/styles';

const stats = [
  { value: '500+', label: 'Academias', sublabel: 'confían en Academix' },
  { value: '50,000+', label: 'Estudiantes', sublabel: 'gestionados cada mes' },
  { value: '98%', label: 'Satisfacción', sublabel: 'en encuestas NPS' },
  { value: '10+', label: 'Países', sublabel: 'en Latinoamérica' },
];

export default function StatsSection() {
  return (
    <Box component="section" sx={{ py: { xs: 10, md: 14 } }}>
      <Container maxWidth="lg">
        <Grid container spacing={3}>
          {stats.map(({ value, label, sublabel }) => (
            <Grid key={label} size={{ xs: 6, md: 3 }}>
              <Paper
                elevation={0}
                sx={{
                  p: { xs: 3, md: 4 },
                  textAlign: 'center',
                  borderRadius: 3,
                  bgcolor: alpha('#1e1e2e', 0.4),
                  border: `1px solid ${alpha('#94a3b8', 0.06)}`,
                  transition: 'all 0.3s',
                  '&:hover': {
                    bgcolor: alpha('#1e1e2e', 0.6),
                    border: `1px solid ${alpha('#10b981', 0.2)}`,
                  },
                }}
              >
                <Stack spacing={0.5}>
                  <Typography
                    variant="h2"
                    sx={{
                      background: 'linear-gradient(135deg, #10b981 0%, #34d399 100%)',
                      WebkitBackgroundClip: 'text',
                      WebkitTextFillColor: 'transparent',
                      fontSize: { xs: '2rem', md: '2.5rem' },
                      fontWeight: 800,
                    }}
                  >
                    {value}
                  </Typography>
                  <Typography
                    variant="h6"
                    sx={{ color: '#f8fafc', fontWeight: 600 }}
                  >
                    {label}
                  </Typography>
                  <Typography
                    variant="body2"
                    sx={{ color: 'text.secondary', fontSize: '0.8rem' }}
                  >
                    {sublabel}
                  </Typography>
                </Stack>
              </Paper>
            </Grid>
          ))}
        </Grid>
      </Container>
    </Box>
  );
}
