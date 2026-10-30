/** Fake-device scenarios, each served by its own backend on its own port. */
export const SCENARIOS = [
  'playing-with-queue',
  'cast-without-queue',
  'long-title',
  'no-artwork',
  'hi-res',
  'paused',
  'stopped',
  'offline',
] as const

export type Scenario = (typeof SCENARIOS)[number]

const FIRST_PORT = 18_101

export function portFor(scenario: Scenario): number {
  return FIRST_PORT + SCENARIOS.indexOf(scenario)
}
