import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Stack from '@mui/material/Stack';
import Divider from '@mui/material/Divider';
import { alpha } from '@mui/material/styles';
import SchoolIcon from '@mui/icons-material/School';
import MusicNoteIcon from '@mui/icons-material/MusicNote';
import SportsIcon from '@mui/icons-material/Sports';
import BrushIcon from '@mui/icons-material/Brush';
import TranslateIcon from '@mui/icons-material/Translate';
import ComputerIcon from '@mui/icons-material/Computer';

const trustedBy = [
  { icon: SchoolIcon, name: 'Academias Bilingües' },
  { icon: MusicNoteIcon, name: 'Escuelas de Música' },
  { icon: SportsIcon, name: 'Centros Deportivos' },
  { icon: BrushIcon, name: 'Escuelas de Arte' },
  { icon: TranslateIcon, name: 'Institutos de Idiomas' },
  { icon: ComputerIcon, name: 'Centros de Tecnología' },
];

export default function SocialProofSection() {
  return (
    <Box component="section" sx={{ py: { xs: 6, md: 8 } }}>
      <Container maxWidth="lg">
        <Divider sx={{ mb: 6 }} />

        <Typography
          variant="body2"
          sx={{
            textAlign: 'center',
            color: 'text.secondary',
            mb: 4,
            fontWeight: 500,
            textTransform: 'uppercase',
            letterSpacing: '0.1em',
            fontSize: '0.75rem',
          }}
        >
          La plataforma elegida por +500 instituciones educativas
        </Typography>

        <Stack
          sx={{ flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'center', alignItems: 'center', gap: { xs: 3, md: 5 } }}
        >
          {trustedBy.map(({ icon: Icon, name }) => (
            <Stack
              key={name}
              sx={{
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                color: alpha('#94a3b8', 0.6),
                transition: 'color 0.2s',
                '&:hover': { color: '#10b981' },
              }}
            >
              <Icon sx={{ fontSize: 20 }} />
              <Typography variant="body2" sx={{ fontWeight: 500, whiteSpace: 'nowrap', color: 'inherit' }}>
                {name}
              </Typography>
            </Stack>
          ))}
        </Stack>

        {/* Scrolling trust metrics */}
        <Stack
          sx={{ flexDirection: 'row', justifyContent: 'center', flexWrap: 'wrap', mt: 5, gap: { xs: 2, md: 4 } }}
        >
          {[
            'G2 Líder en gestión académica 2025',
            '50,000+ estudiantes gestionados',
            '+500 academias en 10 países',
            '98% satisfacción del cliente',
          ].map((text) => (
            <Typography
              key={text}
              variant="body2"
              sx={{
                color: alpha('#94a3b8', 0.5),
                fontSize: '0.8rem',
                fontWeight: 500,
              }}
            >
              {text}
            </Typography>
          ))}
        </Stack>

        <Divider sx={{ mt: 6 }} />
      </Container>
    </Box>
  );
}
