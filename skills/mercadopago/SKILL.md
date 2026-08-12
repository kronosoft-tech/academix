# Mercado Pago — Colombia (Checkout Pro)

## Overview
Mercado Pago Colombia supports Checkout Pro (redirect to MP page) and Checkout Bricks (embedded modules). Colombia does NOT support the preapproval/subscriptions API.

## Checkout Pro (Recommended)
Creates a payment preference and redirects user to MP checkout page.

### Server-side: Create Preference
```
POST https://api.mercadopago.com/checkout/preferences
Authorization: Bearer ACCESS_TOKEN
Content-Type: application/json

{
  "items": [{
    "title": "Academix Pro - Suscripción mensual",
    "quantity": 1,
    "unit_price": 149900,
    "currency_id": "COP"
  }],
  "payer": {
    "email": "user@example.com"
  },
  "external_reference": "user-123-pro-uuid",
  "back_urls": {
    "success": "https://mysite.com/dashboard?payment=success",
    "failure": "https://mysite.com/pricing?payment=failed",
    "pending": "https://mysite.com/dashboard?payment=pending"
  },
  "auto_return": "approved",
  "notification_url": "https://mysite.com/api/webhooks/mercadopago"
}
```

### Response
```json
{
  "id": "3462866096-07c8c47c-73aa-411e-...",
  "init_point": "https://www.mercadopago.com.co/checkout/v1/redirect?pref_id=...",
  "sandbox_init_point": "https://sandbox.mercadopago.com.co/checkout/v1/redirect?pref_id=..."
}
```

### Client-side: Redirect
- Production: `window.location.href = response.init_point`
- Sandbox (TEST- token): `window.location.href = response.sandbox_init_point`

## Webhooks (IPN / Notifications)
MP sends POST to your `notification_url`:
```json
{
  "action": "payment.created",
  "data": { "id": "PAYMENT_ID" }
}
```

Or legacy IPN: `GET /webhook?topic=payment&id=PAYMENT_ID`

### Verify Payment
```
GET https://api.mercadopago.com/v1/payments/{PAYMENT_ID}
Authorization: Bearer ACCESS_TOKEN
```

Response includes: `status` ("approved", "pending", "rejected"), `external_reference`, `transaction_amount`

## Environment Variables
```
MP_ACCESS_TOKEN=TEST-... | APP_USR-...
MP_API_URL=https://api.mercadopago.com
```

## Key Rules
- API route for preferences: `/checkout/preferences` (NOT `/v1/preferences`)
- Payments verification: `/v1/payments/{id}`
- Use `sandbox_init_point` when token starts with `TEST-`
- Use `init_point` when token starts with `APP_USR-` (production)
- Colombia does NOT support `preapproval` (subscriptions). Use Checkout Pro.
- `auto_return: "approved"` makes MP redirect back automatically on approved payments
- `external_reference` is used to match the payment with your internal records
- MP adds `?payment_id=XXX&status=approved&external_reference=YYY` to the success back_url
