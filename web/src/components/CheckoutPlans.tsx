import { useState } from 'react';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import CircularProgress from '@mui/material/CircularProgress';
import ToggleButton from '@mui/material/ToggleButton';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';
import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import CheckIcon from '@mui/icons-material/Check';
import ThemeProvider from './landing/ThemeProvider';
import { PLANS, type Plan } from '../data/plans';
import { geoToGateway, type Gateway } from '../lib/payments/gateway';

interface Props {
  countryCode: string;
}

const GATEWAY_LABELS: Record<Gateway, string> = {
  stripe: 'Tarjeta internacional',
  wompi: 'Bancolombia / Nequi / PSE',
  mercadopago: 'Mercado Pago',
};

function formatCOP(amount: number): string {
  return new Intl.NumberFormat('es-CO', {
    style: 'currency',
    currency: 'COP',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(amount);
}

export default function CheckoutPlans({ countryCode }: Props) {
  const defaultGateway = geoToGateway(countryCode);
  const [gateway, setGateway] = useState<Gateway>(defaultGateway);
  const [loadingPlan, setLoadingPlan] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGatewayChange = (
    _event: React.MouseEvent<HTMLElement>,
    newGateway: Gateway | null,
  ) => {
    if (newGateway) {
      setGateway(newGateway);
    }
  };

  const handleSubscribe = async (plan: Plan) => {
    setError(null);
    setLoadingPlan(plan.id);

    try {
      const response = await fetch(`/api/checkout/${gateway}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ planId: plan.id }),
      });

      if (response.status === 401) {
        window.location.href = '/auth/login?redirect=/pricing';
        return;
      }

      if (!response.ok) {
        const data = await response.json().catch(() => null);
        throw new Error(data?.error || 'Error al crear la sesión de pago');
      }

      const data = await response.json();

      if (data.url) {
        window.location.href = data.url;
      } else if (data.widgetToken) {
        // Wompi widget handling — future implementation
        setError('Widget de pago no disponible aún. Intenta con otra pasarela.');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error inesperado');
    } finally {
      setLoadingPlan(null);
    }
  };

  return (
    <ThemeProvider>
      <Box sx={{ width: '100%' }}>
        <Stack sx={{ alignItems: 'center', mb: 4 }}>
          <ToggleButtonGroup
            value={gateway}
            exclusive
            onChange={handleGatewayChange}
            aria-label="Seleccionar pasarela de pago"
            size="small"
            sx={{
              '& .MuiToggleButton-root': {
                color: 'text.secondary',
                borderColor: 'divider',
                px: 2,
                py: 1,
                fontSize: '0.8rem',
                '&.Mui-selected': {
                  color: 'primary.main',
                  borderColor: 'primary.main',
                  bgcolor: 'rgba(16, 185, 129, 0.08)',
                },
              },
            }}
          >
            <ToggleButton value="stripe">
              {GATEWAY_LABELS.stripe}
            </ToggleButton>
            <ToggleButton value="wompi">
              {GATEWAY_LABELS.wompi}
            </ToggleButton>
            <ToggleButton value="mercadopago">
              {GATEWAY_LABELS.mercadopago}
            </ToggleButton>
          </ToggleButtonGroup>
        </Stack>

        {error && (
          <Alert severity="error" sx={{ mb: 3 }} onClose={() => setError(null)}>
            {error}
          </Alert>
        )}

        <Stack
          sx={{
            flexDirection: { xs: 'column', md: 'row' },
            gap: 3,
            alignItems: 'stretch',
          }}
        >
          {PLANS.map((plan) => {
            const isRecommended = plan.id === 'pro';
            const isLoading = loadingPlan === plan.id;

            return (
              <Card
                key={plan.id}
                sx={{
                  flex: 1,
                  display: 'flex',
                  flexDirection: 'column',
                  position: 'relative',
                  ...(isRecommended && {
                    borderColor: 'primary.main',
                    borderWidth: 2,
                    borderStyle: 'solid',
                  }),
                }}
              >
                <CardContent
                  sx={{
                    display: 'flex',
                    flexDirection: 'column',
                    flexGrow: 1,
                    p: 4,
                  }}
                >
                  <Stack
                    sx={{
                      flexDirection: 'row',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      mb: 2,
                    }}
                  >
                    <Typography variant="h5" component="h3">
                      {plan.name}
                    </Typography>
                    {isRecommended && (
                      <Chip
                        label="Recomendado"
                        color="primary"
                        size="small"
                      />
                    )}
                  </Stack>

                  <Box sx={{ mb: 3 }}>
                    <Typography
                      variant="h3"
                      component="span"
                      sx={{ color: 'text.primary', fontWeight: 700 }}
                    >
                      {formatCOP(plan.priceCOP)}
                    </Typography>
                    <Typography
                      variant="body2"
                      component="span"
                      sx={{ ml: 1 }}
                    >
                      / mes
                    </Typography>
                  </Box>

                  <Stack sx={{ gap: 1.5, flexGrow: 1, mb: 3 }}>
                    {plan.features.map((feature) => (
                      <Stack
                        key={feature}
                        sx={{ flexDirection: 'row', alignItems: 'center', gap: 1 }}
                      >
                        <CheckIcon
                          sx={{ fontSize: 18, color: 'primary.main' }}
                        />
                        <Typography variant="body2" sx={{ color: 'text.secondary' }}>
                          {feature}
                        </Typography>
                      </Stack>
                    ))}
                  </Stack>

                  <Button
                    variant={isRecommended ? 'contained' : 'outlined'}
                    color="primary"
                    fullWidth
                    disabled={isLoading || loadingPlan !== null}
                    onClick={() => handleSubscribe(plan)}
                    aria-label={`Suscribirse al plan ${plan.name}`}
                    sx={{ mt: 'auto' }}
                  >
                    {isLoading ? (
                      <CircularProgress size={24} color="inherit" />
                    ) : (
                      isRecommended ? 'Suscribirse' : 'Comenzar'
                    )}
                  </Button>
                </CardContent>
              </Card>
            );
          })}
        </Stack>
      </Box>
    </ThemeProvider>
  );
}
