import Box from '@mui/material/Box';
import ThemeProvider from './ThemeProvider';
import HeroSection from './HeroSection';
import SocialProofSection from './SocialProofSection';
import FeaturesSection from './FeaturesSection';
import TestimonialsSection from './TestimonialsSection';
import StatsSection from './StatsSection';
import CTASection from './CTASection';
import Footer from './Footer';

/**
 * Static landing sections (no interactivity, no client JS).
 * The interactive Navbar is hydrated separately by the page as an island.
 */
export default function LandingPage() {
  return (
    <ThemeProvider>
      <Box sx={{ minHeight: '100vh', bgcolor: 'background.default' }}>
        <main>
          <HeroSection />
          <SocialProofSection />
          <FeaturesSection />
          <StatsSection />
          <TestimonialsSection />
          <CTASection />
        </main>
        <Footer />
      </Box>
    </ThemeProvider>
  );
}
