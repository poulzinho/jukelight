import { Clock } from './Clock'

export function IdleScreen({ deviceName }: { deviceName: string | null }) {
  return (
    <main className="grid h-full place-items-center text-center motion-safe:animate-fade-in">
      <div>
        <Clock />
        <p className="mt-[1vmin] text-idle text-kiosk-text/55">
          {deviceName ? `${deviceName} · nothing playing` : 'Nothing playing'}
        </p>
      </div>
    </main>
  )
}
