import { useEffect, useState } from 'react'
import type { Player } from './api'
import { predictElapsed } from './time'

/**
 * The current time in milliseconds, refreshed when the predicted elapsed
 * time reaches its next whole second, so the progress moves once a second,
 * in step with the time shown. Nothing is scheduled while paused, without
 * a position, or at the end of the track.
 */
export function useSecondTicks(
  position: Player['position'],
  state: Player['state'],
  clockOffsetMs: number,
): number {
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (!position || state !== 'playing') return
    const elapsed = predictElapsed(position, state, Date.now() + clockOffsetMs)
    if (position.duration !== null && elapsed >= position.duration) return
    // A few milliseconds past the second, so it has surely been reached.
    const wait = (Math.floor(elapsed) + 1 - elapsed) * 1000 + 5
    const timer = setTimeout(() => setNow(Date.now()), wait)
    return () => clearTimeout(timer)
  }, [position, state, clockOffsetMs, now])

  return now
}
