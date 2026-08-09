import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Stack from '@mui/material/Stack';
import Chip from '@mui/material/Chip';
import { alpha, keyframes } from '@mui/material/styles';
import AutoAwesomeIcon from '@mui/icons-material/AutoAwesome';
import ArrowForwardIcon from '@mui/icons-material/ArrowForward';
import DashboardIcon from '@mui/icons-material/Dashboard';
import PeopleIcon from '@mui/icons-material/People';
import MenuBookIcon from '@mui/icons-material/MenuBook';
import GroupsIcon from '@mui/icons-material/Groups';
import PaymentsIcon from '@mui/icons-material/Payments';
import EventNoteIcon from '@mui/icons-material/EventNote';
import AccountBalanceIcon from '@mui/icons-material/AccountBalance';
import PersonIcon from '@mui/icons-material/Person';

const gradientMove = keyframes`
  0% { background-position: 0% 50%; }
  50% { background-position: 100% 50%; }
  100% { background-position: 0% 50%; }
`;

const float = keyframes`
  0%, 100% { transform: translateY(0px); }
  50% { transform: translateY(-8px); }
`;

const sidebarItems = [
  { icon: DashboardIcon, label: 'Dashboard', active: true },
  { icon: PeopleIcon, label: 'Estudiantes', active: false },
  { icon: MenuBookIcon, label: 'Cursos', active: false },
  { icon: GroupsIcon, label: 'Grupos', active: false },
  { icon: PaymentsIcon, label: 'Pagos', active: false },
  { icon: EventNoteIcon, label: 'Asistencia', active: false },
  { icon: AccountBalanceIcon, label: 'Contabilidad', active: false },
  { icon: PersonIcon, label: 'Usuarios', active: false },
];

const statCards = [
  { label: 'Estudiantes', value: '1,247', color: '#3b82f6' },
  { label: 'Cursos', value: '38', color: '#10b981' },
  { label: 'Grupos', value: '12', color: '#8b5cf6' },
  { label: 'Pagos Pend.', value: '5', color: '#f59e0b' },
];

export default function HeroSection() {
  return (
    <Box
      component="section"
      sx={{
        position: 'relative',
        overflow: 'hidden',
        pt: { xs: 10, md: 16 },
        pb: { xs: 10, md: 14 },
      }}
    >
      {/* Animated gradient background */}
      <Box
        sx={{
          position: 'absolute',
          inset: 0,
          background: `linear-gradient(
            -45deg,
            ${alpha('#10b981', 0.12)},
            ${alpha('#8b5cf6', 0.08)},
            ${alpha('#3b82f6', 0.06)},
            ${alpha('#10b981', 0.1)}
          )`,
          backgroundSize: '400% 400%',
          animation: `${gradientMove} 12s ease infinite`,
        }}
      />

      {/* Grid pattern overlay */}
      <Box
        sx={{
          position: 'absolute',
          inset: 0,
          backgroundImage: `
            linear-gradient(${alpha('#94a3b8', 0.04)} 1px, transparent 1px),
            linear-gradient(90deg, ${alpha('#94a3b8', 0.04)} 1px, transparent 1px)
          `,
          backgroundSize: '48px 48px',
        }}
      />

      {/* Radial glow spots */}
      <Box
        sx={{
          position: 'absolute',
          top: '10%',
          left: '20%',
          width: 400,
          height: 400,
          borderRadius: '50%',
          background: `radial-gradient(circle, ${alpha('#10b981', 0.15)} 0%, transparent 70%)`,
          filter: 'blur(60px)',
          animation: `${float} 6s ease-in-out infinite`,
        }}
      />
      <Box
        sx={{
          position: 'absolute',
          bottom: '20%',
          right: '15%',
          width: 300,
          height: 300,
          borderRadius: '50%',
          background: `radial-gradient(circle, ${alpha('#8b5cf6', 0.12)} 0%, transparent 70%)`,
          filter: 'blur(50px)',
          animation: `${float} 8s ease-in-out infinite 2s`,
        }}
      />

      <Container maxWidth="lg" sx={{ position: 'relative', zIndex: 1 }}>
        <Stack sx={{ alignItems: 'center', gap: 4 }}>
          <Chip
            icon={<AutoAwesomeIcon sx={{ fontSize: 16 }} />}
            label="Plataforma #1 para academias en Latinoamérica"
            variant="outlined"
            sx={{
              borderColor: alpha('#10b981', 0.3),
              color: '#10b981',
              bgcolor: alpha('#10b981', 0.05),
              px: 1,
              '& .MuiChip-icon': { color: '#10b981' },
            }}
          />

          <Typography
            variant="h1"
            component="h1"
            sx={{
              textAlign: 'center',
              maxWidth: 900,
              background: 'linear-gradient(135deg, #f8fafc 0%, #cbd5e1 50%, #94a3b8 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
            }}
          >
            La gestión académica no debería ser complicada.
          </Typography>

          <Typography
            variant="body1"
            sx={{
              textAlign: 'center',
              maxWidth: 680,
              fontSize: { xs: '1rem', md: '1.25rem' },
              color: 'text.secondary',
              lineHeight: 1.8,
            }}
          >
            Unifica estudiantes, cursos, pagos y reportes en un solo espacio.
            Diseñado para que tu equipo se enfoque en enseñar, no en administrar.
          </Typography>

          <Stack sx={{ flexDirection: { xs: 'column', sm: 'row' }, gap: 2, pt: 2 }}>
            <Button
              variant="contained"
              size="large"
              href="/downloads"
              endIcon={<ArrowForwardIcon />}
              sx={{ px: 4, py: 1.5 }}
            >
              Comenzar gratis
            </Button>
            <Button
              variant="outlined"
              size="large"
              href="/pricing"
              sx={{ px: 4, py: 1.5 }}
            >
              Ver planes
            </Button>
          </Stack>

          {/* Realistic product mockup */}
          <Box
            sx={{
              mt: { xs: 6, md: 10 },
              width: '100%',
              maxWidth: 1100,
              borderRadius: 3,
              border: `1px solid ${alpha('#94a3b8', 0.12)}`,
              bgcolor: alpha('#0f0f14', 0.9),
              backdropFilter: 'blur(20px)',
              overflow: 'hidden',
              boxShadow: `
                0 32px 64px ${alpha('#000', 0.5)},
                0 0 0 1px ${alpha('#10b981', 0.08)},
                inset 0 1px 0 ${alpha('#fff', 0.03)}
              `,
              animation: `${float} 6s ease-in-out infinite`,
            }}
          >
            {/* Window chrome */}
            <Box sx={{
              px: 2, py: 1.5,
              borderBottom: `1px solid ${alpha('#94a3b8', 0.08)}`,
              display: 'flex', alignItems: 'center', gap: 1,
              bgcolor: alpha('#1e1e2e', 0.5),
            }}>
              <Box sx={{ width: 12, height: 12, borderRadius: '50%', bgcolor: '#ef4444' }} />
              <Box sx={{ width: 12, height: 12, borderRadius: '50%', bgcolor: '#eab308' }} />
              <Box sx={{ width: 12, height: 12, borderRadius: '50%', bgcolor: '#22c55e' }} />
              <Typography
                variant="body2"
                sx={{ ml: 2, fontSize: '0.7rem', color: alpha('#94a3b8', 0.5) }}
              >
                Academix — Dashboard
              </Typography>
            </Box>

            {/* App content */}
            <Box sx={{ display: 'flex', minHeight: { xs: 280, md: 420 } }}>
              {/* Sidebar */}
              <Box sx={{
                width: { xs: 48, md: 200 },
                borderRight: `1px solid ${alpha('#94a3b8', 0.08)}`,
                py: 2,
                display: 'flex', flexDirection: 'column',
              }}>
                {/* Logo */}
                <Typography sx={{
                  px: { xs: 1, md: 2.5 },
                  mb: 2, fontWeight: 800, fontSize: { xs: '0.6rem', md: '0.9rem' },
                  color: '#10b981',
                }}>
                  <Box component="span" sx={{ display: { xs: 'none', md: 'inline' } }}>Academix</Box>
                  <Box component="span" sx={{ display: { xs: 'inline', md: 'none' } }}>A</Box>
                </Typography>
                {/* Nav items */}
                {sidebarItems.map(({ icon: Icon, label, active }) => (
                  <Box key={label} sx={{
                    mx: { xs: 0.5, md: 1 },
                    px: { xs: 1, md: 1.5 },
                    py: 0.8,
                    borderRadius: 1.5,
                    display: 'flex', alignItems: 'center', gap: 1.5,
                    bgcolor: active ? alpha('#10b981', 0.1) : 'transparent',
                    border: active ? `1px solid ${alpha('#10b981', 0.2)}` : '1px solid transparent',
                  }}>
                    <Icon sx={{ fontSize: { xs: 14, md: 16 }, color: active ? '#10b981' : alpha('#94a3b8', 0.6) }} />
                    <Typography sx={{
                      fontSize: '0.75rem', fontWeight: active ? 600 : 400,
                      color: active ? '#10b981' : alpha('#94a3b8', 0.7),
                      display: { xs: 'none', md: 'block' },
                    }}>
                      {label}
                    </Typography>
                  </Box>
                ))}
                {/* User at bottom */}
                <Box sx={{ mt: 'auto', mx: 1, p: 1, display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Box sx={{
                    width: 28, height: 28, borderRadius: '50%',
                    bgcolor: alpha('#10b981', 0.15),
                    display: 'flex', alignItems: 'center', justifyContent: 'center',
                  }}>
                    <Typography sx={{ fontSize: '0.65rem', color: '#10b981', fontWeight: 600 }}>AD</Typography>
                  </Box>
                  <Box sx={{ display: { xs: 'none', md: 'block' } }}>
                    <Typography sx={{ fontSize: '0.7rem', color: '#f8fafc', fontWeight: 500 }}>Admin</Typography>
                    <Typography sx={{ fontSize: '0.6rem', color: alpha('#94a3b8', 0.6) }}>Administrador</Typography>
                  </Box>
                </Box>
              </Box>

              {/* Main dashboard content */}
              <Box sx={{ flex: 1, p: { xs: 2, md: 3 } }}>
                {/* Title */}
                <Typography sx={{
                  fontSize: { xs: '0.85rem', md: '1.1rem' },
                  fontWeight: 700, color: '#f8fafc', mb: 2,
                }}>
                  Dashboard
                </Typography>

                {/* Stat cards row */}
                <Stack direction="row" spacing={{ xs: 1, md: 2 }} sx={{ mb: { xs: 2, md: 3 } }}>
                  {statCards.map(({ label, value, color }) => (
                    <Box key={label} sx={{
                      flex: 1,
                      p: { xs: 1, md: 2 },
                      borderRadius: 2,
                      bgcolor: alpha(color, 0.06),
                      border: `1px solid ${alpha(color, 0.15)}`,
                    }}>
                      <Typography sx={{
                        fontSize: { xs: '0.55rem', md: '0.7rem' },
                        color: alpha('#94a3b8', 0.7), mb: 0.5,
                      }}>
                        {label}
                      </Typography>
                      <Typography sx={{
                        fontSize: { xs: '0.9rem', md: '1.3rem' },
                        fontWeight: 700, color: '#f8fafc',
                      }}>
                        {value}
                      </Typography>
                    </Box>
                  ))}
                </Stack>

                {/* Quick actions + chart area */}
                <Stack direction="row" spacing={{ xs: 1, md: 2 }} sx={{ height: { xs: 100, md: 180 } }}>
                  {/* Chart placeholder */}
                  <Box sx={{
                    flex: 2,
                    borderRadius: 2,
                    bgcolor: alpha('#1e1e2e', 0.5),
                    border: `1px solid ${alpha('#94a3b8', 0.08)}`,
                    p: 2,
                    display: 'flex', flexDirection: 'column',
                  }}>
                    <Typography sx={{
                      fontSize: '0.65rem', color: alpha('#94a3b8', 0.6), mb: 1,
                    }}>
                      Ingresos mensuales
                    </Typography>
                    {/* Fake chart bars */}
                    <Stack sx={{ flexDirection: 'row', alignItems: 'flex-end', gap: 0.5, flex: 1 }}>
                      {[40, 65, 45, 80, 55, 70, 90, 60, 75, 85, 50, 95].map((h, i) => (
                        <Box key={i} sx={{
                          flex: 1, height: `${h}%`, borderRadius: 0.5,
                          bgcolor: i === 11
                            ? '#10b981'
                            : alpha('#10b981', 0.2 + (h / 200)),
                        }} />
                      ))}
                    </Stack>
                  </Box>
                  {/* Quick actions */}
                  <Box sx={{
                    flex: 1,
                    borderRadius: 2,
                    bgcolor: alpha('#1e1e2e', 0.5),
                    border: `1px solid ${alpha('#94a3b8', 0.08)}`,
                    p: 1.5,
                    display: { xs: 'none', md: 'flex' }, flexDirection: 'column', gap: 1,
                  }}>
                    <Typography sx={{ fontSize: '0.65rem', color: alpha('#94a3b8', 0.6) }}>
                      Acciones rápidas
                    </Typography>
                    {['Registrar Estudiante', 'Crear Grupo', 'Registrar Pago'].map((action) => (
                      <Box key={action} sx={{
                        py: 0.8, px: 1.5, borderRadius: 1,
                        bgcolor: alpha('#94a3b8', 0.05),
                        border: `1px solid ${alpha('#94a3b8', 0.08)}`,
                      }}>
                        <Typography sx={{ fontSize: '0.6rem', color: alpha('#f8fafc', 0.7) }}>
                          {action}
                        </Typography>
                      </Box>
                    ))}
                  </Box>
                </Stack>
              </Box>
            </Box>
          </Box>
        </Stack>
      </Container>
    </Box>
  );
}
