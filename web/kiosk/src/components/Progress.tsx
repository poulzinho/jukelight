import type { Player } from '../api'
import { formatTime, predictElapsed } from '../time'
import { useSecondTicks } from '../useSecondTicks'

interface Props {
  position: NonNullable<Player['position']>
  state: Player['state']
  clockOffsetMs: number
}

/**
 * Advances by itself between updates, once a second together with the
 * elapsed time, without a transition: between seconds nothing animates, so
 * the browser can stop drawing. Moves with `transform` only.
 */
export function Progress({ position, state, clockOffsetMs }: Props) {
  const now = useSecondTicks(position, state, clockOffsetMs) + clockOffsetMs
  const elapsed = predictElapsed(position, state, now)
  const { duration } = position
  const fraction = duration !== null && duration > 0 ? Math.floor(elapsed) / duration : 0

  return (
    <div className="mt-[3vmin]">
      <div
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={position.duration ?? undefined}
        aria-valuenow={Math.floor(elapsed)}
        className="h-[0.9vmin] overflow-hidden rounded-full bg-kiosk-text/20"
      >
        <div
          className="h-full origin-left rounded-full bg-kiosk-accent"
          style={{ transform: `scaleX(${fraction})` }}
        />
      </div>
      <div className="mt-[1.2vmin] flex justify-between text-time text-kiosk-text/60 tabular-nums">
        <span>{formatTime(elapsed)}</span>
        <span>{position.duration === null ? '' : formatTime(position.duration)}</span>
      </div>
    </div>
  )
}

/**
 * The room the progress takes, for a track without a position: in portrait
 * it keeps the signal path and "Up next" in place; in landscape it takes none.
 */
export function ProgressSlot() {
  return (
    <div aria-hidden="true" className="hidden portrait:mt-[3vmin] portrait:block">
      <div className="h-[0.9vmin]" />
      <div className="mt-[1.2vmin] text-time">&nbsp;</div>
    </div>
  )
}
