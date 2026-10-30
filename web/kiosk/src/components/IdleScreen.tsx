import { useNow } from '../useNow'

export function IdleScreen({ deviceName }: { deviceName: string | null }) {
  const now = useNow(1000)
  const time = new Date(now).toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })
  return (
    <main className="grid h-full place-items-center text-center motion-safe:animate-fade-in">
      <div>
        <div role="timer" className="text-clock font-extralight text-kiosk-text/80 tabular-nums">
          {time}
        </div>
        <p className="mt-[1vmin] text-idle text-kiosk-text/55">
          {deviceName ? `${deviceName} · nothing playing` : 'Nothing playing'}
        </p>
      </div>
    </main>
  )
}
