import type { Change, Player, Quality, QualityClass, Signal, Speaker } from '../api'

/** What the signal diagram shows, built from one snapshot. */
export interface DiagramModel {
  source: SourceNode
  /** Music Assistant, when it is the route and reachable. */
  route: { name: string; logoUrl: string | null } | null
  /** What the source streams, shown by its logo; `null` when not reported. */
  streams: { quality: string; short: string; codec: string } | null
  speakers: SpeakerRow[]
  streamer: StreamerCard
  outputs: OutputNode[]
  /** The class of the source's line. */
  sourceClass: QualityClass
  /** The class of the lines after the route, and of the streamer. */
  playerClass: QualityClass
  /** Whether the lines after the route use the lighter shade. */
  soft: boolean
}

export interface SourceNode {
  name: string
  brand: string | null
  logoUrl: string | null
  /** The logo's width over its height, once it has loaded. */
  logoAspect?: number
  /** Music Assistant itself, unreachable: its logo greyed with "unavailable". */
  unavailable: boolean
}

export interface SpeakerRow {
  name: string
  kind: Speaker['kind']
  playing: boolean
  /** "16/44.1" or "256 kbps"; empty when not reported. */
  quality: string
  /** "PCM", "FLAC", "AAC"…; empty when not reported. */
  codec: string
  class: QualityClass
}

export interface StreamerCard {
  /** "24 bit / 96 kHz", "320 kbps", or what is known ("AirPlay"). */
  main: string
  /** The codec and what changed on the way; empty when nothing is known. */
  codec: string
  room: string
  hiRes: boolean
  hiResLogoUrl: string | null
  markUrl: string | null
}

export interface OutputNode {
  name: string
  kind: 'speaker' | 'subwoofer'
}

export function diagramModel(player: Player): DiagramModel | null {
  const { source, signal } = player
  if (!signal) return null
  const via = source?.via ?? null
  // Music Assistant unreachable and itself the source: its node says it all.
  const unavailable = Boolean(via && !via.available && source?.brand === via.brand && source?.name === via.name)
  const route = via && via.available ? { name: via.name, logoUrl: via.logoUrl } : null
  const known = signal.class !== 'unknown' && signal.source !== null
  const delivered = signal.delivered ?? signal.source
  return {
    source: {
      name: source?.name ?? player.session.label,
      brand: source?.brand ?? null,
      logoUrl: source?.logoUrl ?? null,
      unavailable,
    },
    route,
    streams: known && signal.source ? { quality: long(signal.source), short: short(signal.source), codec: codecName(signal.source) } : null,
    speakers: player.speakers.map((speaker) => ({
      name: speaker.name,
      kind: speaker.kind,
      playing: speaker.playing,
      quality: speaker.quality ? short(speaker.quality) : '',
      codec: speaker.quality ? codecName(speaker.quality) : '',
      class: speaker.quality?.class ?? 'unknown',
    })),
    streamer: {
      main: known && delivered ? long(delivered) : unavailable ? player.device.name : (source?.name ?? player.session.label),
      codec: known && delivered ? codecLine(signal, delivered) : '',
      room: unavailable ? '' : player.device.name,
      hiRes: signal.class === 'hi-res',
      hiResLogoUrl: signal.hiResLogoUrl,
      markUrl: player.device.markUrl,
    },
    outputs: player.outputs,
    sourceClass: signal.class,
    playerClass: signal.class,
    soft: route !== null,
  }
}

/** "24 bit / 96 kHz", "320 kbps", or the label. */
export function long(quality: Quality): string {
  if (quality.bitDepth && quality.sampleRateHz) return `${quality.bitDepth} bit / ${kilohertz(quality.sampleRateHz)} kHz`
  if (quality.bitrate) return `${Math.round(quality.bitrate / 1000)} kbps`
  return quality.label
}

/** "24/96", "320 kbps", or the label. */
export function short(quality: Quality): string {
  if (quality.bitDepth && quality.sampleRateHz) return `${quality.bitDepth}/${kilohertz(quality.sampleRateHz)}`
  if (quality.bitrate) return `${Math.round(quality.bitrate / 1000)} kbps`
  return quality.label
}

/** The codec, or "lossy" for a bitrate without one. */
function codecName(quality: Quality): string {
  return quality.codec ?? (quality.bitrate ? 'lossy' : '')
}

/**
 * The streamer's second line: the codec, then what changed on the way
 * ("FLAC · resampled from 16 / 44.1"), "PCM · decoded from 320 kbps" when a
 * lossy source arrives as raw samples, or "untouched".
 */
function codecLine(signal: Signal, delivered: Quality): string {
  const parts = [codecName(delivered)]
  const source = signal.source
  if (source?.bitrate && delivered.codec === 'PCM') parts.push(`decoded from ${short(source)}`)
  for (const change of signal.changes) parts.push(describe(change, signal))
  if (parts.length === 1) parts.push('untouched')
  return parts.filter(Boolean).join(' · ')
}

function describe(change: Change, { source, delivered }: Signal): string {
  switch (change.kind) {
    case 'resampled':
      return source?.bitDepth && source.sampleRateHz
        ? `resampled from ${source.bitDepth} / ${kilohertz(source.sampleRateHz)}`
        : 'resampled'
    case 'bitDepth':
      return delivered?.bitDepth && source?.bitDepth ? `${delivered.bitDepth} bit from ${source.bitDepth}` : 'bit depth changed'
    case 'normalised':
      return change.gainDb === null ? 'normalised' : `normalised (${decibels(change.gainDb)})`
    case 'dsp':
      return 'DSP'
  }
}

/** 96000 -> "96", 44100 -> "44.1", 22050 -> "22.05". */
function kilohertz(hz: number): string {
  return String(Number((hz / 1000).toFixed(2)))
}

/** −6.2 -> "−6.2 dB", 2 -> "+2 dB", with a real minus sign. */
function decibels(gain: number): string {
  const value = String(Number(Math.abs(gain).toFixed(1)))
  return `${gain < 0 ? '−' : '+'}${value} dB`
}
