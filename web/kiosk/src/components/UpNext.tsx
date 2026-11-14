import type { Player } from '../api'

/** At most five entries: in portrait as many as fit above the page's bottom margin, four in landscape. */
const SHOWN = 5

/**
 * Each entry keyed by its title and artists, so an entry keeps its element
 * as the queue moves on; a repeated entry gets a number.
 */
function keyed(entries: NonNullable<Player['queue']>) {
  const seen = new Map<string, number>()
  return entries.map((entry) => {
    const name = `${entry.title}|${entry.artists.join(', ')}`
    const count = seen.get(name) ?? 0
    seen.set(name, count + 1)
    return { entry, key: `${name}|${count}` }
  })
}

export function UpNext({ queue }: { queue: NonNullable<Player['queue']> }) {
  if (queue.length === 0) return null
  return (
    // Beside the signal diagram, its heading level with the diagram's top. In
    // portrait the list fills the room down to the page's bottom margin; entries that don't
    // fit wrap into a second, clipped column, so none shows cut in half.
    <section className="min-w-0 text-[length:initial] portrait:flex portrait:min-h-0 portrait:flex-col portrait:self-stretch">
      <h2 id="up-next" className="mb-[1.6vmin] text-badge font-semibold tracking-[0.14em] text-kiosk-text/50 uppercase">
        Up next
      </h2>
      <ol
        aria-labelledby="up-next"
        className="grid gap-[1.4vmin] portrait:flex portrait:min-h-0 portrait:flex-1 portrait:flex-col portrait:flex-wrap portrait:content-start portrait:overflow-hidden"
      >
        {keyed(queue.slice(0, SHOWN)).map(({ entry, key }, index) => (
          <li
            key={key}
            className={`grid min-w-0 grid-cols-[6vmin_1fr] items-center gap-[1.8vmin] portrait:w-full ${index === SHOWN - 1 ? 'landscape:hidden' : ''}`}
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
