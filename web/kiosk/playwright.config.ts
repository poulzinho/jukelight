import { defineConfig } from '@playwright/test'

/**
 * End-to-end screenshots: the real backend binary with fake scenarios,
 * at 4K in landscape and portrait, with reduced motion.
 *
 * Build first: `npm run build` (kiosk), then `cargo build` (embeds it).
 */
export default defineConfig({
  testDir: './e2e',
  globalSetup: './e2e/global-setup.ts',
  fullyParallel: true,
  reporter: [['list']],
  use: {
    reducedMotion: 'reduce',
    deviceScaleFactor: 1,
  },
  projects: [
    { name: 'landscape', use: { viewport: { width: 3840, height: 2160 } } },
    { name: 'portrait', use: { viewport: { width: 2160, height: 3840 } } },
  ],
})
