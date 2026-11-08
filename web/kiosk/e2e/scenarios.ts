/** Fake-device scenarios, each served by its own backend on its own port. */
export const SCENARIOS = [
  'playing-with-queue',
  'cast-without-queue',
  'long-title',
  'no-album',
  'no-artwork',
  'hi-res',
  'album-colours',
  'optical-input',
  'paused',
  'stopped',
  'offline',
  'music-assistant',
  'music-assistant-unavailable',
  'long-badges',
] as const

/**
 * Scenarios whose backend gets the test logos (plain shapes, never real
 * logos); the others show the plain badges.
 */
export const WITH_LOGOS: readonly Scenario[] = ['music-assistant']

export type Scenario = (typeof SCENARIOS)[number]

const FIRST_PORT = 18_101

export function portFor(scenario: Scenario): number {
  return FIRST_PORT + SCENARIOS.indexOf(scenario)
}
