import { Disc3, UserRound, Users } from 'lucide-react'
import { useLayoutEffect, useRef, useState } from 'react'
import type { Player } from '../api'
import { useFontsLoaded } from '../useFontsLoaded'

/**
 * Title sizes in vmin, largest first: a title takes the first at which it
 * fits. One line at the two largest, two lines at smaller sizes, three at
 * the smallest; only a title that doesn't fit even then ends with "…". In
 * portrait the title's room is one line at the largest size, which two
 * lines at 5.4 or 4.6 and three at 3.6 also fit, so nothing below moves.
 */
const STEPS = [
  { size: 7.2, lines: 1 },
  { size: 6.2, lines: 1 },
  { size: 5.4, lines: 2 },
  { size: 4.6, lines: 2 },
  { size: 3.6, lines: 3 },
] as const

/** Words that name a group, as whole words, in any case. */
const GROUP_WORDS =
  /\b(orchestra|philharmonic|ensemble|quartet|trio|choir|band|orquesta|orchestre|orchester|coro|chœur|chor|cuarteto|quatuor|quartett|quartetto)\b/i

/** Several artists, or one whose name says it is a group. */
const isGroup = (artists: string[]) => artists.length > 1 || GROUP_WORDS.test(artists[0] ?? '')

function Title({ title }: { title: string }) {
  const ref = useRef<HTMLHeadingElement>(null)
  const fontsLoaded = useFontsLoaded()
  const [fitted, setFitted] = useState({ title, fontsLoaded, step: 0 })
  // A new title, or newly loaded fonts, start again from the largest size.
  const step = fitted.title === title && fitted.fontsLoaded === fontsLoaded ? fitted.step : 0
  useLayoutEffect(() => {
    const element = ref.current
    if (!element || step >= STEPS.length - 1) return
    // Letters may overhang their line a little: a fifth of a line is still a fit.
    const overhang = (Number.parseFloat(getComputedStyle(element).fontSize) || 0) * 0.2
    if (element.scrollHeight > element.clientHeight + overhang) {
      setFitted({ title, fontsLoaded, step: step + 1 })
    }
  }, [title, fontsLoaded, step])
  const { size, lines } = STEPS[step] ?? STEPS[0]
  return (
    <div className="portrait:h-[11.2vmin]">
      <h1
        ref={ref}
        className="overflow-hidden font-bold tracking-tight [display:-webkit-box] [-webkit-box-orient:vertical]"
        style={{ fontSize: `${size}vmin`, lineHeight: 1.02, WebkitLineClamp: lines }}
      >
        {title}
      </h1>
    </div>
  )
}

export function TrackInfo({ track }: { track: NonNullable<Player['track']> }) {
  const group = isGroup(track.artists)
  const ArtistIcon = group ? Users : UserRound
  return (
    <div className="flex min-w-0 flex-col gap-[1.6vmin] portrait:gap-[1.1vmin]">
      <Title title={track.title} />
      {track.artists.length > 0 ? (
        // Artists may take a second line: the one place the portrait layout moves.
        <p className="flex min-w-0 items-start gap-[1.2vmin] text-artist leading-[1.15] font-medium text-kiosk-text/85">
          <ArtistIcon
            data-testid="artist-icon"
            data-kind={group ? 'group' : 'person'}
            aria-hidden="true"
            className="mt-[0.24em] size-[0.72em] shrink-0 opacity-55"
            strokeWidth={2}
          />
          <span className="line-clamp-2">{track.artists.join(', ')}</span>
        </p>
      ) : (
        <p aria-hidden="true" className="hidden text-artist portrait:block portrait:min-h-[1lh]" />
      )}
      {track.album ? (
        <p className="flex min-w-0 items-center gap-[1.2vmin] text-album text-kiosk-text/60">
          <Disc3 data-testid="album-icon" aria-hidden="true" className="size-[0.85em] shrink-0 opacity-70" strokeWidth={2} />
          <span className="truncate">{track.album}</span>
        </p>
      ) : (
        <p aria-hidden="true" className="hidden text-album portrait:block portrait:min-h-[1lh]" />
      )}
    </div>
  )
}
