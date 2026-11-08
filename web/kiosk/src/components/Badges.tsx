import type { Player } from '../api'

const badge = 'rounded-full px-[1.6vmin] py-[0.7vmin] text-badge font-semibold tracking-wide'

export function Badges({ player }: { player: Player }) {
  const { session, quality } = player
  return (
    // One row in portrait, so a long badge never pushes "Up next" down.
    <div className="mt-[2vmin] flex flex-wrap items-center gap-[1.2vmin] portrait:flex-nowrap portrait:overflow-hidden">
      {session.label && (
        <span
          className={`${badge} ${session.available ? 'bg-kiosk-text/10 text-kiosk-text/85' : 'border-[0.2vmin] border-kiosk-text/25 text-kiosk-text/60'}`}
        >
          {session.available ? session.label : `${session.label} unavailable`}
        </span>
      )}
      {quality?.hiRes && (
        <span className={`${badge} bg-linear-to-b from-[rgb(236_196_96)] to-[rgb(184_137_43)] uppercase text-[rgb(26_20_5)]`}>
          HI-RES
        </span>
      )}
      {quality && (
        <span className={`${badge} bg-kiosk-accent/25 text-kiosk-text`}>
          {[quality.label, quality.detail].filter(Boolean).join(' · ')}
        </span>
      )}
    </div>
  )
}
