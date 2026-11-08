/** The published API (v1), as served by `/api/v1/player`. */

export type PlaybackState = 'playing' | 'paused' | 'stopped' | 'buffering'

export interface Player {
  online: boolean
  deviceName: string
  state: PlaybackState
  session: { label: string; available: boolean }
  track: { title: string; artists: string[]; album: string | null } | null
  /** The large version of the track's cover. */
  artUrl: string | null
  /** A tiny version of the same cover, to stretch over the screen. */
  backdropUrl: string | null
  /** `elapsed` and `duration` in seconds, `observedAt` in epoch milliseconds. */
  position: { elapsed: number; duration: number | null; observedAt: number } | null
  quality: Quality | null
  /** `null` when the session has no queue. */
  queue: { title: string; artists: string[]; artUrl: string | null }[] | null
  /** Album colours as `[red, green, blue]`; `null` when unknown. */
  palette: { background: Rgb; accent: Rgb } | null
  /** The audio input while one plays; `startedAt` (epoch ms) is `null` while waiting for a signal. */
  input: { kind: 'optical' | 'analog' | 'hdmi-arc' | 'other'; name: string; startedAt: number | null } | null
  /** Where the music comes from; `null` when nothing is known. */
  source: Source | null
  /** How the music reaches the player; `null` when idle or on an input. */
  signal: Signal | null
  /** The player itself: its maker's brand and its name. */
  device: Brand & { name: string }
}

export interface Quality {
  label: string
  detail: string | null
  hiRes: boolean
  /** The user's Hi-Res logo, for a Hi-Res quality; `null` otherwise. */
  hiResLogoUrl: string | null
  /** For a lossless format; `null` for a lossy one. */
  codec: string | null
  bitDepth: number | null
  sampleRateHz: number | null
  /** Bits per second, for a lossy stream; `null` otherwise. */
  bitrate: number | null
}

/** The lowest quality along the path. */
export type QualityClass = 'hi-res' | 'lossless' | 'lossy' | 'unknown'

/** One thing the route changed on the way, in this order. */
export type Change =
  | { kind: 'resampled' }
  | { kind: 'bitDepth' }
  | { kind: 'normalised'; gainDb: number | null }
  | { kind: 'dsp' }

/** The path from the source, through the route, to the player. */
export interface Signal {
  class: QualityClass
  /** The quality as the source delivers it. */
  source: Quality | null
  /** What the route sends to the player; `null` without a route or when not reported. */
  delivered: Quality | null
  changes: Change[]
  /** The user's Hi-Res logo, when the whole path is Hi-Res. */
  hiResLogoUrl: string | null
}

export type Rgb = [number, number, number]

export const PLAYER_URL = '/api/v1/player'
export const EVENTS_URL = '/api/v1/player/events'

/**
 * A brand, as a short identifier such as `qobuz` (`null` when unknown), and
 * the user's logo for it (`null` without one).
 */
export interface Brand {
  brand: string | null
  logoUrl: string | null
}

/** A music service or another source, and the route the music takes. */
export interface Source extends Brand {
  name: string
  via: (Brand & { name: string; available: boolean }) | null
}
