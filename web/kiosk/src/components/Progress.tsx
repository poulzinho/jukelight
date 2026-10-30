import type { Player } from '../api'
import { formatTime, predictElapsed } from '../time'
import { useNow } from '../useNow'

interface Props {
  position: NonNullable<Player['position']>
  state: Player['state']
  clockOffsetMs: number
}

/** Advances by itself between updates; moves with `transform` only. */
export function Progress({ position, state, clockOffsetMs }: Props) {
  const now = useNow(250) + clockOffsetMs
  const elapsed = predictElapsed(position, state, now)
  const fraction = position.duration ? elapsed / position.duration : 0

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
          className="h-full origin-left rounded-full bg-kiosk-accent motion-safe:transition-transform motion-safe:duration-300 motion-safe:ease-linear"
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
