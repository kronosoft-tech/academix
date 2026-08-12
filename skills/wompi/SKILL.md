# Wompi Payment Gateway — Colombia

## Overview
Wompi is a Colombian payment gateway by Bancolombia. Supports: Credit/Debit cards, PSE, Nequi, Bancolombia Transfer, Efecty.

## Integration Methods

### 1. Widget (Recommended for SPA/Islands)
Opens a popup overlay inside your page. Works on localhost.

```javascript
// Load script in <head>
<script src="https://checkout.wompi.co/widget.js"></script>

// Open checkout
var checkout = new WidgetCheckout({
  currency: 'COP',
  amountInCents: 4990000, // $49,900 COP
  reference: 'unique-reference-123',
  publicKey: 'pub_test_XXX',
  signature: { integrity: 'sha256-hash-here' },
  // redirectUrl: 'https://...' // OPTIONAL — only HTTPS domains
});

checkout.open(function(result) {
  var transaction = result.transaction;
  console.log("Status:", transaction.status); // APPROVED, DECLINED, VOIDED, ERROR
  console.log("ID:", transaction.id);
});
```

### 2. Web Checkout (Form redirect)
Redirects to checkout.wompi.co. Does NOT work on localhost (403).

```html
<form action="https://checkout.wompi.co/p/" method="GET">
  <input type="hidden" name="public-key" value="pub_test_XXX" />
  <input type="hidden" name="currency" value="COP" />
  <input type="hidden" name="amount-in-cents" value="4990000" />
  <input type="hidden" name="reference" value="unique-ref" />
  <input type="hidden" name="signature:integrity" value="sha256-hash" />
  <input type="hidden" name="redirect-url" value="https://mysite.com/callback" />
  <button type="submit">Pay</button>
</form>
```

## Integrity Signature
SHA256 of: `<reference><amountInCents><currency><integritySecret>`

```javascript
const concatenated = `${reference}${amountInCents}${currency}${WOMPI_INTEGRITY_SECRET}`;
const hash = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(concatenated));
const integrity = Array.from(new Uint8Array(hash)).map(b => b.toString(16).padStart(2, '0')).join('');
```

## Webhooks (Events)
- URL configured in Dashboard → Developers → Events
- POST to your URL with `event: "transaction.updated"`
- Verify checksum: SHA256 of concatenated `signature.properties` values + `timestamp` + events_secret
- Must respond HTTP 200. Retries: 30min, 3h, 24h.

## Environment Variables
```
WOMPI_PUBLIC_KEY=pub_test_... | pub_prod_...
WOMPI_PRIVATE_KEY=prv_test_... | prv_prod_...
WOMPI_INTEGRITY_SECRET=test_integrity_... | prod_integrity_...
WOMPI_EVENTS_SECRET=test_events_... | prod_events_...
WOMPI_API_URL=https://sandbox.wompi.co/v1 | https://production.wompi.co/v1
```

## Key Rules
- Amounts are in CENTAVOS (COP cents): $49,900 = 4990000
- `redirectUrl` in Widget is OPTIONAL and only works with HTTPS
- Web Checkout does NOT work on localhost (CloudFront blocks non-HTTPS referers)
- Widget JS (`WidgetCheckout`) DOES work on localhost
- Always validate webhook checksum before processing
- References must be unique per transaction
