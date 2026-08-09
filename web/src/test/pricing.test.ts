import { describe, it, expect } from 'vitest';
import pricingData from '../data/pricing.json';

function lookupPricing(countryCode: string) {
  const countryPricing = pricingData.find((p) => p.country_code === countryCode);
  const usdFallback = pricingData.find((p) => p.country_code === 'US')!;
  return countryPricing || usdFallback;
}

describe('Pricing lookup', () => {
  it('should return COP prices for Colombia (CO)', () => {
    const pricing = lookupPricing('CO');
    expect(pricing.currency_code).toBe('COP');
    expect(pricing.prices.basic).toBe(89900);
    expect(pricing.prices.pro).toBe(149900);
    expect(pricing.prices.premium).toBe(259900);
  });

  it('should return MXN prices for Mexico (MX)', () => {
    const pricing = lookupPricing('MX');
    expect(pricing.currency_code).toBe('MXN');
    expect(pricing.prices.basic).toBe(449);
    expect(pricing.prices.pro).toBe(799);
    expect(pricing.prices.premium).toBe(1399);
  });

  it('should return CLP prices for Chile (CL)', () => {
    const pricing = lookupPricing('CL');
    expect(pricing.currency_code).toBe('CLP');
    expect(pricing.prices.basic).toBe(22990);
    expect(pricing.prices.pro).toBe(41990);
    expect(pricing.prices.premium).toBe(73990);
  });

  it('should return ARS prices for Argentina (AR)', () => {
    const pricing = lookupPricing('AR');
    expect(pricing.currency_code).toBe('ARS');
    expect(pricing.prices.basic).toBe(35900);
    expect(pricing.prices.pro).toBe(64900);
    expect(pricing.prices.premium).toBe(114900);
  });

  it('should return USD for US and dollarized countries', () => {
    const pricing = lookupPricing('US');
    expect(pricing.currency_code).toBe('USD');
    expect(pricing.prices.basic).toBe(29);
    expect(pricing.prices.pro).toBe(49);
    expect(pricing.prices.premium).toBe(79);

    // Ecuador, Panamá, El Salvador, Venezuela also use USD
    expect(lookupPricing('EC').currency_code).toBe('USD');
    expect(lookupPricing('PA').currency_code).toBe('USD');
    expect(lookupPricing('SV').currency_code).toBe('USD');
  });

  it('should return USD fallback for unknown country code', () => {
    const pricing = lookupPricing('ZZ');
    expect(pricing.currency_code).toBe('USD');
    expect(pricing.display_name).toBe('Internacional');
    expect(pricing.prices.basic).toBe(29);
  });

  it('should have all LatAm countries covered', () => {
    const countries = pricingData.map((p) => p.country_code);
    expect(countries).toContain('CO');
    expect(countries).toContain('MX');
    expect(countries).toContain('AR');
    expect(countries).toContain('CL');
    expect(countries).toContain('PE');
    expect(countries).toContain('BR');
    expect(countries).toContain('EC');
    expect(countries).toContain('CR');
    expect(countries).toContain('DO');
    expect(countries.length).toBeGreaterThanOrEqual(15);
  });

  it('all prices should use psychological pricing (end in 9 or 0)', () => {
    for (const country of pricingData) {
      const { basic, pro, premium } = country.prices;
      // Prices should end in 9 or 0 (psychological pricing)
      for (const price of [basic, pro, premium]) {
        const lastDigit = price % 10;
        expect(
          lastDigit === 9 || lastDigit === 0,
          `${country.country_code} price ${price} doesn't end in 9 or 0`
        ).toBe(true);
      }
    }
  });
});
