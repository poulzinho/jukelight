/** The published API (v1), as served by `/api/v1/player`. */

export type PlaybackState = 'playing' | 'paused' | 'stopped' | 'buffering'

export interface Player {
  online: boolean
  deviceName: string
  state: PlaybackState
  session: { label: string; available: boolean }
  track: { title: string; artists: string[]; album: string | null } | null
  artUrl: string | null
  /** `elapsed` and `duration` in seconds, `observedAt` in epoch milliseconds. */
  position: { elapsed: number; duration: number | null; observedAt: number } | null
  quality: { label: string; detail: string | null; hiRes: boolean } | null
  /** `null` when the session has no queue. */
  queue: { title: string; artists: string[]; artUrl: string | null }[] | null
  /** Album colours as `[red, green, blue]`; `null` when unknown. */
  palette: { background: Rgb; accent: Rgb } | null
}

export type Rgb = [number, number, number]

export const PLAYER_URL = '/api/v1/player'
export const EVENTS_URL = '/api/v1/player/events'
