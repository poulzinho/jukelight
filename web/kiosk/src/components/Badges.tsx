import { Airplay, AudioLines, Bluetooth, Cable, Folder, Radio, Server } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import type { Player, Source } from '../api'

/** Brands whose logo already shows the name: no name next to it. */
const WORDMARKS = new Set(['qobuz', 'spotify', 'tidal', 'apple-music', 'deezer', 'plex'])

/**
 * Each logo's height, relative to the badge text, so different logos look
 * equally large. Tuned on screenshots with the reference host's logos.
 */
const LOGO_SCALE: Record<string, number> = {
  qobuz: 1.1,
  spotify: 1.35,
  tidal: 0.75,
  'apple-music': 0.95,
  deezer: 1.35,
  plex: 1.05,
  'music-assistant': 1.3,
  navidrome: 1.25,
  jellyfin: 1.25,
}

/** An icon for a source without a logo: generic shapes, not brand logos. */
function SourceIcon({ source }: { source: { brand: string | null; name: string } }) {
  const props = { 'aria-hidden': true, className: 'size-[1.15em] shrink-0' } as const
  switch (source.brand) {
    case 'subsonic':
      return <Server {...props} />
    case 'library':
      return <Folder {...props} />
    case 'airplay':
      return <Airplay {...props} />
    case 'bluetooth':
      return <Bluetooth {...props} />
  }
  if (source.name === 'Stream') return <Radio {...props} />
  if (/input|HDMI ARC/i.test(source.name)) return <Cable {...props} />
  return <AudioLines {...props} />
}

/** A brand's logo at its own height (in `em`, relative to the badge text). */
function Logo({ brand, url, name, grey = false }: { brand: string | null; url: string; name: string; grey?: boolean }) {
  const scale = (brand && LOGO_SCALE[brand]) || 1.1
  return (
    <img
      src={url}
      alt={name}
      style={{ height: `${scale}em` }}
      className={`block w-auto shrink-0 ${grey ? 'opacity-50 grayscale' : ''}`}
    />
  )
}

const badge =
  'inline-flex h-[1.85em] shrink-0 items-center gap-[0.45em] rounded-full px-[0.7em] font-semibold'

/** The source: its logo, or an icon and its name; then "via" the route. */
function SourceBadge({ source, withVia }: { source: Source; withVia: boolean }) {
  const via = withVia && source.via?.available ? source.via : null
  return (
    <span
      data-testid="source-badge"
      className={`${badge} bg-black/40 text-kiosk-text shadow-[inset_0_0_0_0.06em_rgb(255_255_255/0.14)]`}
    >
      {source.logoUrl ? (
        <Logo brand={source.brand} url={source.logoUrl} name={source.name} />
      ) : (
        <SourceIcon source={source} />
      )}
      {(!source.logoUrl || !WORDMARKS.has(source.brand ?? '')) && <span>{source.name}</span>}
      {via && (
        <>
          <span className="font-medium text-kiosk-text/60">via</span>
          {via.logoUrl ? <Logo brand={via.brand} url={via.logoUrl} name={via.name} /> : <span>{via.name}</span>}
        </>
      )}
    </span>
  )
}

/** A route that can't be reached: greyed and outlined. */
function RouteUnavailable({ via }: { via: NonNullable<Source['via']> }) {
  return (
    <span
      data-testid="route-unavailable"
      className={`${badge} font-medium text-kiosk-text/60 shadow-[inset_0_0_0_0.08em_rgb(245_245_247/0.3)]`}
    >
      {via.logoUrl ? (
        <>
          <Logo brand={via.brand} url={via.logoUrl} name={via.name} grey />
          <span>unavailable</span>
        </>
      ) : (
        <span>{via.name} unavailable</span>
      )}
    </span>
  )
}

/** "24 bit / 96 kHz" and the codec, or "160 kbps"; the label otherwise. */
function qualityText(quality: NonNullable<Player['quality']>): { numbers: string; codec: string | null } {
  if (quality.bitDepth && quality.sampleRateHz) {
    const khz = Number((quality.sampleRateHz / 1000).toFixed(1))
    return { numbers: `${quality.bitDepth} bit / ${khz} kHz`, codec: quality.codec }
  }
  if (quality.bitrate) return { numbers: `${Math.round(quality.bitrate / 1000)} kbps`, codec: null }
  return { numbers: quality.label, codec: null }
}

/** Below this width (in `em`), what's left of the detail isn't worth showing. */
const MIN_DETAIL_EM = 2

/**
 * The source badge, the Hi-Res mark, the quality and its detail, on one
 * line: nothing wraps. When it doesn't fit, the detail is shortened ("…"),
 * then left out when hardly any of it is left, then "via" goes as a whole;
 * the source, the Hi-Res mark and the quality always show.
 */
export function Badges({ player }: { player: Player }) {
  const { source, quality } = player
  // A line with other content starts fitting afresh.
  const via = source?.via
  const content = [
    source?.brand,
    source?.name,
    source?.logoUrl,
    via?.name,
    via?.available,
    via?.logoUrl,
    quality?.hiRes,
    quality?.hiResLogoUrl,
    quality && qualityText(quality).numbers,
    quality?.codec,
    quality?.detail,
  ].join('|')
  return <BadgeLine key={content} source={source} quality={quality} />
}

function BadgeLine({ source, quality }: Pick<Player, 'source' | 'quality'>) {
  const row = useRef<HTMLDivElement>(null)
  const detail = useRef<HTMLSpanElement>(null)
  const [withDetail, setWithDetail] = useState(true)
  const [withVia, setWithVia] = useState(true)

  useEffect(() => {
    const element = row.current
    if (!element || typeof ResizeObserver === 'undefined') return
    let width = element.clientWidth
    const fit = () => {
      // More room (for example a rotated screen): try everything again.
      if (element.clientWidth > width + 1) {
        width = element.clientWidth
        setWithVia(true)
        setWithDetail(true)
        return
      }
      width = element.clientWidth
      const em = parseFloat(getComputedStyle(element).fontSize)
      if (detail.current && detail.current.clientWidth < MIN_DETAIL_EM * em) setWithDetail(false)
      else if (withVia && element.scrollWidth > element.clientWidth + 1) {
        // Without "via", the detail may fit again: try it once more.
        setWithVia(false)
        setWithDetail(true)
      }
    }
    const observer = new ResizeObserver(fit)
    observer.observe(element)
    if (detail.current) observer.observe(detail.current)
    return () => observer.disconnect()
  }, [withDetail, withVia])

  const via = source?.via
  const unavailable = via && !via.available ? via : null
  // When the route itself is the source (it can't be reached), its greyed
  // badge says it all.
  const sourceIsRoute = unavailable && source?.brand === unavailable.brand && source?.name === unavailable.name
  const text = quality && qualityText(quality)
  return (
    <div
      ref={row}
      className="mt-[2vmin] flex items-center gap-[0.45em] overflow-hidden whitespace-nowrap text-quality"
    >
      {unavailable && <RouteUnavailable via={unavailable} />}
      {source && !sourceIsRoute && <SourceBadge source={source} withVia={withVia} />}
      {quality?.hiRes &&
        (quality.hiResLogoUrl ? (
          <img src={quality.hiResLogoUrl} alt="Hi-Res" className="block h-[1.85em] w-auto shrink-0" />
        ) : (
          <span className="shrink-0 rounded-full bg-linear-to-b from-[rgb(236_196_96)] to-[rgb(184_137_43)] px-[0.55em] py-[0.3em] text-[0.75em] font-bold tracking-[0.08em] text-[rgb(26_20_5)] [font-feature-settings:'case']">
            HI-RES
          </span>
        ))}
      {text && (
        <span className="shrink-0 font-semibold tabular-nums">
          {text.numbers}
          {text.codec && <span className="ml-[0.35em] font-medium text-kiosk-text/70">{text.codec}</span>}
        </span>
      )}
      {quality?.detail && withDetail && (
        <>
          <span aria-hidden="true" className="shrink-0 text-kiosk-text/30">
            ·
          </span>
          <span ref={detail} className="min-w-0 truncate font-medium text-kiosk-text/50">
            {quality.detail}
          </span>
        </>
      )}
    </div>
  )
}
