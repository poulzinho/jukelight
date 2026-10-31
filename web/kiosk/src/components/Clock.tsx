import { useNow } from '../useNow'

/** The current time, large; shared by the idle and the input screen. */
export function Clock() {
  const now = useNow(1000)
  const time = new Date(now).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })
  return (
    <div role="timer" className="text-clock font-extralight text-kiosk-text/80 tabular-nums">
      {time}
    </div>
  )
}
