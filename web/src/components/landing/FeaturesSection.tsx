import Box from '@mui/material/Box';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Stack from '@mui/material/Stack';
import Chip from '@mui/material/Chip';
import { alpha } from '@mui/material/styles';
import PersonAddIcon from '@mui/icons-material/PersonAdd';
import CalendarMonthIcon from '@mui/icons-material/CalendarMonth';
import PaymentsIcon from '@mui/icons-material/Payments';
import BarChartIcon from '@mui/icons-material/BarChart';
import GroupsIcon from '@mui/icons-material/Groups';
import NotificationsActiveIcon from '@mui/icons-material/NotificationsActive';
import SchoolIcon from '@mui/icons-material/School';
import ReceiptLongIcon from '@mui/icons-material/ReceiptLong';
import type { SvgIconComponent } from '@mui/icons-material';

export default function FeaturesSection() {
  return (
    <Box component="section" id="features" sx={{ py: { xs: 10, md: 16 } }}>
      <Container maxWidth="lg">
        <Stack sx={{ alignItems: 'center', gap: 2, mb: { xs: 6, md: 10 } }}>
          <Chip
            label="Funcionalidades"
            size="small"
            sx={{
              bgcolor: alpha('#10b981', 0.08),
              color: '#10b981',
              border: `1px solid ${alpha('#10b981', 0.2)}`,
              fontWeight: 600,
            }}
          />
          <Typography variant="h2" sx={{ textAlign: 'center' }}>
            Todo lo que tu academia necesita.
          </Typography>
          <Typography
            variant="body1"
            sx={{ maxWidth: 600, color: 'text.secondary', textAlign: 'center' }}
          >
            Una plataforma completa diseñada junto a directores de academia reales.
            No es un ERP genérico — es TU herramienta.
          </Typography>
        </Stack>

        {/* Bento Grid — 8 cols × 8 rows, pure MUI sx */}
        <Box sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'repeat(8, 1fr)' },
          gridTemplateRows: { md: 'repeat(8, 80px)' },
          gap: 2,
        }}>

          {/* 1: Matriculación — col-span-3 row-span-3 */}
          <Box sx={{ gridColumn: { md: 'span 3' }, gridRow: { md: 'span 3' } }}>
            <BentoCard icon={PersonAddIcon} color="#10b981" tag="Captura"
              title="Inscripciones sin fricción."
              description="Formularios inteligentes, documentación digital y expedientes automáticos."
            >
              <Box sx={{ mt: 2, p: 2, borderRadius: 2, bgcolor: alpha('#10b981', 0.04), border: `1px solid ${alpha('#10b981', 0.1)}` }}>
                <Stack sx={{ gap: 1.5 }}>
                  <Stack sx={{ flexDirection: 'row', gap: 1.5 }}>
                    <MockInput label="Nombre completo" width="60%" />
                    <MockInput label="DNI / Cédula" width="40%" />
                  </Stack>
                  <Stack sx={{ flexDirection: 'row', gap: 1.5 }}>
                    <MockInput label="Email" width="50%" />
                    <MockInput label="Teléfono" width="30%" />
                    <MockInput label="Curso" width="20%" />
                  </Stack>
                  <Box sx={{ width: 120, py: 0.8, px: 2, borderRadius: 1.5, textAlign: 'center', bgcolor: '#10b981', mt: 1 }}>
                    <Typography sx={{ fontSize: '0.65rem', color: '#fff', fontWeight: 600 }}>Matricular</Typography>
                  </Box>
                </Stack>
              </Box>
            </BentoCard>
          </Box>

          {/* 2: Cursos — col-span-3 row-span-3 col-start-4 */}
          <Box sx={{ gridColumn: { md: '4 / span 3' }, gridRow: { md: 'span 3' } }}>
            <BentoCard icon={CalendarMonthIcon} color="#3b82f6" tag="Organiza"
              title="Cursos y horarios claros."
              description="Asigna profesores, configura horarios y gestiona capacidad en segundos."
            >
              <Stack sx={{ gap: 1, mt: 2 }}>
                {[
                  { course: 'Inglés B2', time: 'L-M-V 9:00', spots: '18/20' },
                  { course: 'Piano Básico', time: 'M-J 15:00', spots: '8/10' },
                  { course: 'React Avanzado', time: 'S 10:00', spots: '12/15' },
                ].map(({ course, time, spots }) => (
                  <Stack key={course} sx={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', py: 0.8, px: 1.5, borderRadius: 1.5, bgcolor: alpha('#3b82f6', 0.04), border: `1px solid ${alpha('#3b82f6', 0.08)}` }}>
                    <Box>
                      <Typography sx={{ fontSize: '0.7rem', color: alpha('#f8fafc', 0.9), fontWeight: 500 }}>{course}</Typography>
                      <Typography sx={{ fontSize: '0.6rem', color: alpha('#94a3b8', 0.7) }}>{time}</Typography>
                    </Box>
                    <Chip label={spots} size="small" sx={{ height: 18, fontSize: '0.55rem', bgcolor: alpha('#3b82f6', 0.1), color: '#3b82f6', border: `1px solid ${alpha('#3b82f6', 0.2)}` }} />
                  </Stack>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 3: Asistencia — col-span-2 row-span-6 col-start-7 */}
          <Box sx={{ gridColumn: { md: '7 / span 2' }, gridRow: { md: 'span 6' } }}>
            <BentoCard icon={SchoolIcon} color="#8b5cf6" tag="Asistencia"
              title="Control diario con un tap."
              description="Lista de asistencia digital. Detecta patrones y notifica."
            >
              <Stack sx={{ gap: 1, mt: 2 }}>
                {['Ana García', 'Carlos López', 'María Ruiz', 'Juan Pérez', 'Sofía Díaz', 'Luis Morales', 'Elena Torres'].map((name, i) => (
                  <Stack key={name} sx={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', py: 0.8, px: 1.5, borderRadius: 1.5, bgcolor: alpha('#8b5cf6', 0.04), border: `1px solid ${alpha('#8b5cf6', 0.08)}` }}>
                    <Stack sx={{ flexDirection: 'row', alignItems: 'center', gap: 1 }}>
                      <Box sx={{ width: 24, height: 24, borderRadius: '50%', bgcolor: alpha('#8b5cf6', 0.15), display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                        <Typography sx={{ fontSize: '0.5rem', color: '#8b5cf6', fontWeight: 600 }}>
                          {name.split(' ').map(n => n[0]).join('')}
                        </Typography>
                      </Box>
                      <Typography sx={{ fontSize: '0.68rem', color: alpha('#f8fafc', 0.8) }}>{name}</Typography>
                    </Stack>
                    <Box sx={{ width: 18, height: 18, borderRadius: 0.8, bgcolor: i < 6 ? alpha('#10b981', 0.2) : alpha('#ef4444', 0.2), border: `1.5px solid ${i < 6 ? '#10b981' : '#ef4444'}`, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                      <Typography sx={{ fontSize: '0.55rem', color: i < 6 ? '#10b981' : '#ef4444' }}>
                        {i < 6 ? '✓' : '✗'}
                      </Typography>
                    </Box>
                  </Stack>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 4: Pagos — col-span-2 row-span-3 row-start-4 */}
          <Box sx={{ gridColumn: { md: 'span 2' }, gridRow: { md: '4 / span 3' } }}>
            <BentoCard icon={PaymentsIcon} color="#f59e0b" tag="Pagos"
              title="Cobros en piloto automático."
              description="Cobros recurrentes, recordatorios y recibos digitales."
            >
              <Stack sx={{ gap: 1, mt: 2 }}>
                {[
                  { name: 'Ana García', amount: '$45', status: 'Pagado', ok: true },
                  { name: 'Carlos López', amount: '$45', status: 'Pendiente', ok: false },
                  { name: 'María Ruiz', amount: '$60', status: 'Pagado', ok: true },
                ].map(({ name, amount, status, ok }) => (
                  <Stack key={name} sx={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'space-between', py: 0.7, px: 1.2, borderRadius: 1.5, bgcolor: alpha('#f59e0b', 0.03), border: `1px solid ${alpha('#f59e0b', 0.08)}` }}>
                    <Typography sx={{ fontSize: '0.68rem', color: alpha('#f8fafc', 0.8) }}>{name}</Typography>
                    <Stack sx={{ flexDirection: 'row', gap: 0.8, alignItems: 'center' }}>
                      <Typography sx={{ fontSize: '0.68rem', color: '#f8fafc', fontWeight: 600 }}>{amount}</Typography>
                      <Chip label={status} size="small" sx={{ height: 16, fontSize: '0.5rem', fontWeight: 600, bgcolor: ok ? alpha('#10b981', 0.1) : alpha('#f59e0b', 0.1), color: ok ? '#10b981' : '#f59e0b' }} />
                    </Stack>
                  </Stack>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 5: Reportes — col-span-4 row-span-3 col-start-3 row-start-4 */}
          <Box sx={{ gridColumn: { md: '3 / span 4' }, gridRow: { md: '4 / span 3' } }}>
            <BentoCard icon={BarChartIcon} color="#06b6d4" tag="Reportes"
              title="Métricas en tiempo real."
              description="Dashboards de ingresos, retención y asistencia. Exporta a PDF con un clic."
            >
              <Stack sx={{ flexDirection: 'row', alignItems: 'flex-end', gap: 0.5, mt: 2, flex: 1 }}>
                {[35, 50, 42, 68, 55, 72, 80, 65, 78, 88, 70, 92, 60, 85, 75].map((h, i) => (
                  <Box key={i} sx={{ flex: 1, height: `${h}%`, borderRadius: 0.5, bgcolor: alpha('#06b6d4', 0.15 + (h / 200)) }} />
                ))}
              </Stack>
              <Stack sx={{ flexDirection: 'row', justifyContent: 'space-between', mt: 1 }}>
                {['Ene', 'Feb', 'Mar', 'Abr', 'May', 'Jun', 'Jul', 'Ago', 'Sep', 'Oct', 'Nov', 'Dic'].map((m) => (
                  <Typography key={m} sx={{ fontSize: '0.5rem', color: alpha('#94a3b8', 0.5) }}>{m}</Typography>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 6: Roles — col-span-3 row-span-2 row-start-7 */}
          <Box sx={{ gridColumn: { md: 'span 3' }, gridRow: { md: '7 / span 2' } }}>
            <BentoCard icon={GroupsIcon} color="#ec4899" tag="Roles"
              title="Cada quien ve lo suyo."
              description="Admin, gerente, empleado y profesor con permisos diferenciados."
            >
              <Stack sx={{ gap: 0.8, mt: 1.5 }}>
                {[
                  { role: 'Administrador', access: 'Acceso total', pct: 100 },
                  { role: 'Gerente', access: 'Sin config. sistema', pct: 75 },
                  { role: 'Profesor', access: 'Solo sus grupos', pct: 30 },
                ].map(({ role, access, pct }) => (
                  <Box key={role}>
                    <Stack sx={{ flexDirection: 'row', justifyContent: 'space-between', mb: 0.3 }}>
                      <Typography sx={{ fontSize: '0.68rem', color: alpha('#f8fafc', 0.8), fontWeight: 500 }}>{role}</Typography>
                      <Typography sx={{ fontSize: '0.6rem', color: alpha('#94a3b8', 0.6) }}>{access}</Typography>
                    </Stack>
                    <Box sx={{ height: 4, borderRadius: 2, bgcolor: alpha('#ec4899', 0.1) }}>
                      <Box sx={{ height: '100%', width: `${pct}%`, borderRadius: 2, bgcolor: alpha('#ec4899', 0.5) }} />
                    </Box>
                  </Box>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 7: Alertas — col-span-2 row-span-2 col-start-4 row-start-7 */}
          <Box sx={{ gridColumn: { md: '4 / span 2' }, gridRow: { md: '7 / span 2' } }}>
            <BentoCard icon={NotificationsActiveIcon} color="#f97316" tag="Alertas"
              title="Nunca pierdas un pago."
              description=""
            >
              <Stack sx={{ gap: 0.8, mt: 1.5 }}>
                {[
                  { text: '3 pagos vencidos hoy', type: 'warn' as const },
                  { text: 'Juan Pérez: 3 faltas', type: 'alert' as const },
                  { text: '5 renovaciones pronto', type: 'info' as const },
                ].map(({ text, type }) => (
                  <Box key={text} sx={{
                    py: 0.7, px: 1.2, borderRadius: 1.5,
                    bgcolor: alpha(type === 'warn' ? '#f59e0b' : type === 'alert' ? '#ef4444' : '#3b82f6', 0.06),
                    borderLeft: `3px solid ${type === 'warn' ? '#f59e0b' : type === 'alert' ? '#ef4444' : '#3b82f6'}`,
                  }}>
                    <Typography sx={{ fontSize: '0.68rem', color: alpha('#f8fafc', 0.8) }}>{text}</Typography>
                  </Box>
                ))}
              </Stack>
            </BentoCard>
          </Box>

          {/* 8: Facturación — col-span-3 row-span-2 col-start-6 row-start-7 */}
          <Box sx={{ gridColumn: { md: '6 / span 3' }, gridRow: { md: '7 / span 2' } }}>
            <BentoCard icon={ReceiptLongIcon} color="#10b981" tag="Facturación"
              title="Reportes financieros en segundos."
              description="Exporta PDF, controla ingresos vs gastos."
            >
              <Box sx={{
                mt: 1.5, flex: 1, borderRadius: 2,
                bgcolor: alpha('#10b981', 0.04),
                border: `1px dashed ${alpha('#10b981', 0.2)}`,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
              }}>
                <Typography sx={{ fontSize: '0.7rem', color: alpha('#10b981', 0.5) }}>
                  📊 Screenshot módulo contabilidad
                </Typography>
              </Box>
            </BentoCard>
          </Box>

        </Box>
      </Container>
    </Box>
  );
}

// Reusable Bento Card
interface BentoCardProps {
  icon: SvgIconComponent;
  color: string;
  tag: string;
  title: string;
  description: string;
  children?: React.ReactNode;
}

function BentoCard({ icon: Icon, color, tag, title, description, children }: BentoCardProps) {
  return (
    <Box sx={{
      height: '100%',
      p: { xs: 2.5, md: 3 },
      borderRadius: 3,
      bgcolor: alpha('#1e1e2e', 0.5),
      backdropFilter: 'blur(10px)',
      border: `1px solid ${alpha('#94a3b8', 0.08)}`,
      overflow: 'hidden',
      display: 'flex',
      flexDirection: 'column',
      transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
      '&:hover': {
        border: `1px solid ${alpha(color, 0.3)}`,
        boxShadow: `0 8px 32px ${alpha(color, 0.08)}`,
        transform: 'translateY(-2px)',
      },
    }}>
      <Stack sx={{ flexDirection: 'row', alignItems: 'center', gap: 1.5, mb: 1 }}>
        <Box sx={{
          width: 28, height: 28, borderRadius: 1.5,
          bgcolor: alpha(color, 0.1),
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          flexShrink: 0,
        }}>
          <Icon sx={{ fontSize: 16, color }} />
        </Box>
        <Chip
          label={tag}
          size="small"
          sx={{
            height: 18, fontSize: '0.55rem', fontWeight: 600,
            bgcolor: alpha(color, 0.08), color,
            border: `1px solid ${alpha(color, 0.2)}`,
          }}
        />
      </Stack>
      <Typography sx={{ mb: 0.5, color: '#f8fafc', fontSize: '0.95rem', fontWeight: 600 }}>
        {title}
      </Typography>
      {description && (
        <Typography sx={{ color: 'text.secondary', fontSize: '0.75rem', lineHeight: 1.5 }}>
          {description}
        </Typography>
      )}
      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column' }}>{children}</Box>
    </Box>
  );
}

// Mock input for enrollment card
function MockInput({ label, width }: { label: string; width: string }) {
  return (
    <Box sx={{ width }}>
      <Typography sx={{ fontSize: '0.55rem', color: alpha('#94a3b8', 0.6), mb: 0.3 }}>{label}</Typography>
      <Box sx={{ height: 24, borderRadius: 1, bgcolor: alpha('#94a3b8', 0.06), border: `1px solid ${alpha('#94a3b8', 0.1)}` }} />
    </Box>
  );
}
