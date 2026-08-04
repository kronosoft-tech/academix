import { describe, it, expect } from 'vitest';
import pricingData from '../data/pricing.json';

function lookupPricing(countryCode: string) {
  const countryPricing = pricingData.find((p) => p.country_code === countryCode);
  const usdFallback = {
    symbol: '$',
    currency_code: 'USD',
    display_name: 'Internacional',
    prices: { basic: 20, pro: 30, premium: 60 },
  };
  return countryPricing || usdFallback;
}

describe('Pricing lookup', () => {
  it('should return COP prices for Colombia (CO)', () => {
    const pricing = lookupPricing('CO');
    expect(pricing.currency_code).toBe('COP');
    expect(pricing.prices.basic).toBe(83000);
    expect(pricing.prices.pro).toBe(124500);
    expect(pricing.prices.premium).toBe(249000);
  });

  it('should return MXN prices for Mexico (MX)', () => {
    const pricing = lookupPricing('MX');
    expect(pricing.currency_code).toBe('MXN');
    expect(pricing.prices.basic).toBe(350);
  });

  it('should return CLP prices for Chile (CL)', () => {
    const pricing = lookupPricing('CL');
    expect(pricing.currency_code).toBe('CLP');
    expect(pricing.prices.basic).toBe(19000);
  });

  it('should return ARS prices for Argentina (AR)', () => {
    const pricing = lookupPricing('AR');
    expect(pricing.currency_code).toBe('ARS');
    expect(pricing.prices.basic).toBe(21000);
  });

  it('should return USD fallback for unknown country', () => {
    const pricing = lookupPricing('US');
    expect(pricing.currency_code).toBe('USD');
    expect(pricing.prices.basic).toBe(20);
    expect(pricing.prices.pro).toBe(30);
    expect(pricing.prices.premium).toBe(60);
  });

  it('should return USD fallback for non-existent country code', () => {
    const pricing = lookupPricing('ZZ');
    expect(pricing.currency_code).toBe('USD');
    expect(pricing.display_name).toBe('Internacional');
  });
});
