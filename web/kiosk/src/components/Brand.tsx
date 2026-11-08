import { Airplay, AudioLines, Bluetooth, Cable, Folder, Radio, Server } from 'lucide-react'

/** Brands whose logo already shows the name: no name next to it. */
export const WORDMARKS = new Set(['qobuz', 'spotify', 'tidal', 'apple-music', 'deezer', 'plex'])

/**
 * Each logo's height, relative to the text around it, so different logos look
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
export function SourceIcon({ source }: { source: { brand: string | null; name: string } }) {
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

/** A brand's logo at its own height (in `em`, relative to the text around it). */
export function Logo({ brand, url, name, grey = false }: { brand: string | null; url: string; name: string; grey?: boolean }) {
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
