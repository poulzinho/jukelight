import type { Player } from '../api'
import { type Row, groupQueue } from '../upNext/group'

/** At most five entries: in portrait as many as fit above the page's bottom margin, four in landscape. */
const SHOWN = 5

type EntryRow = Extract<Row, { kind: 'entry' }>

interface Props {
  queue: NonNullable<Player['queue']>
  /** The track playing now: an upcoming movement of its work is grouped under the work. */
  current: { title: string; artists: string[] }
}

export function UpNext({ queue, current }: Props) {
  if (queue.length === 0) return null
  const rows = groupQueue(queue.slice(0, SHOWN), current)
  // Each entry with the heading that leads its group, so the two wrap, and
  // leave the clipped column, as one: a heading never stands alone.
  const items: { entry: EntryRow; heading: string | null }[] = []
  let heading: string | null = null
  for (const row of rows) {
    if (row.kind === 'heading') heading = row.text
    else {
      items.push({ entry: row, heading })
      heading = null
    }
  }
  return (
    // Beside the signal diagram, its heading level with the diagram's top.
    // The list fills the room down to the page's bottom margin; entries that
    // don't fit wrap into a second, clipped column, so none shows cut in half.
    <section className="flex min-h-0 min-w-0 flex-col self-stretch text-[length:initial]">
      <h2 id="up-next" className="mb-[1.6vmin] text-badge font-semibold tracking-[0.14em] text-kiosk-text/50 uppercase">
        Up next
      </h2>
      <ol
        aria-labelledby="up-next"
        className="flex min-h-0 flex-1 flex-col flex-wrap content-start gap-[1.4vmin] overflow-hidden"
      >
        {items.map(({ entry, heading }, index) => (
          <li key={entry.key} className={`w-full min-w-0 ${index === SHOWN - 1 ? 'landscape:hidden' : ''}`}>
            {heading && (
              <p data-part="group-heading" className="mt-[0.6vmin] mb-[1vmin] truncate text-next-artist font-semibold text-kiosk-text/60">
                {heading}
              </p>
            )}
            <div className="grid min-w-0 grid-cols-[6vmin_1fr] items-center gap-[1.8vmin]">
              {entry.artUrl ? (
                <img src={entry.artUrl} alt="" className="size-[6vmin] rounded-[0.6vmin] object-cover" />
              ) : (
                <div className="size-[6vmin] rounded-[0.6vmin] bg-white/10" />
              )}
              <div className="min-w-0">
                <p className="line-clamp-2 text-next-title leading-[1.15] font-semibold">{entry.title}</p>
                {entry.artist && <p className="truncate text-next-artist text-kiosk-text/60">{entry.artist}</p>}
              </div>
            </div>
          </li>
        ))}
      </ol>
    </section>
  )
}
