import type { Player } from '../api'

/** Four entries in landscape, three in portrait. */
const SHOWN = 4

export function UpNext({ queue }: { queue: NonNullable<Player['queue']> }) {
  if (queue.length === 0) return null
  return (
    <section className="mt-[4vmin]">
      <h2 id="up-next" className="mb-[1.6vmin] text-badge font-semibold tracking-[0.14em] text-kiosk-text/50 uppercase">
        Up next
      </h2>
      <ol aria-labelledby="up-next" className="grid gap-[1.4vmin]">
        {queue.slice(0, SHOWN).map((entry, index) => (
          <li
            key={`${index}-${entry.title}`}
            className={`grid min-w-0 grid-cols-[6vmin_1fr] items-center gap-[1.8vmin] ${index === SHOWN - 1 ? 'portrait:hidden' : ''}`}
          >
            {entry.artUrl ? (
              <img src={entry.artUrl} alt="" className="size-[6vmin] rounded-[0.6vmin] object-cover" />
            ) : (
              <div className="size-[6vmin] rounded-[0.6vmin] bg-white/10" />
            )}
            <div className="min-w-0">
              <p className="truncate text-next-title font-semibold">{entry.title}</p>
              <p className="truncate text-next-artist text-kiosk-text/60">{entry.artists.join(', ')}</p>
            </div>
          </li>
        ))}
      </ol>
    </section>
  )
}
