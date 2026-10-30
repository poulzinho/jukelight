import type { Player } from './api'

/** 75 -> "1:15". */
export function formatTime(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds))
  return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`
}

/** "Example Album" -> "EA". */
export function initials(text: string): string {
  return text
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')
}

/**
 * Elapsed seconds expected at `now` (server time): it advances only while
 * playing and never passes the end of the track.
 */
export function predictElapsed(
  position: NonNullable<Player['position']>,
  state: Player['state'],
  now: number,
): number {
  const running = state === 'playing' ? Math.max(0, now - position.observedAt) / 1000 : 0
  const elapsed = position.elapsed + running
  return position.duration === null ? elapsed : Math.min(elapsed, position.duration)
}
