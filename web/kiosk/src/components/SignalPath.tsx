import { useEffect, useRef, useState, type CSSProperties, type RefObject } from 'react'
import type { Change, Player, Quality, Signal, Source } from '../api'
import { Logo, SourceIcon, WORDMARKS } from './Brand'

/**
 * How the music reaches the player, in one row: the source, a wave, the
 * route (Music Assistant) and a second wave when there is one, and the sound
 * box. One colour for the whole row, from the quality class; the wave after
 * the route is a lighter shade. When the row runs out of room, the waves get
 * shorter, then their labels go, then the box's last line is shortened, and
 * last the route's name goes.
 */
export function SignalPath({ player }: { player: Player }) {
  const row = useRef<HTMLDivElement>(null)
  const box = useRef<HTMLDivElement>(null)
  const { source, signal } = player
  const level = useGiveWay(row, box, signal ? soundText(signal).join('|') : '')
  if (!signal) return null

  const via = source?.via ?? null
  // When the route can't be reached and is itself the source, its node says it all.
  const routeIsSource = via && !via.available && source?.brand === via.brand && source?.name === via.name
  const route = via && !routeIsSource ? via : null
  const moving = player.state !== 'paused' && (!via || via.available)
  const known = signal.class !== 'unknown'
  const colours = {
    '--signal': `var(--color-signal-${signal.class})`,
    '--signal-soft': `var(--color-signal-${signal.class}-soft)`,
  } as CSSProperties

  return (
    <div
      ref={row}
      data-testid="signal-path"
      style={colours}
      className="mt-[1.6vmin] flex items-center gap-[0.4em] overflow-hidden whitespace-nowrap text-signal"
    >
      {source && <Node service={source} unavailable={Boolean(routeIsSource)} route={Boolean(routeIsSource)} />}
      <Wave shade="main" moving={moving} label={known ? sourceLabel(signal.source) : null} />
      {route && (
        <>
          <Node service={route} unavailable={!route.available} route iconOnly={level >= 2} />
          <Wave
            shade="soft"
            moving={moving}
            label={known && route.available ? deliveredLabel(signal) : null}
          />
        </>
      )}
      <SoundBox signal={signal} boxRef={box} squeezed={level >= 1} soft={Boolean(route?.available)} />
    </div>
  )
}

/**
 * A source or a route: its logo, or an icon and its name. A route's logo
 * stands alone, as a wordmark's does.
 */
function Node({
  service,
  unavailable,
  route = false,
  iconOnly = false,
}: {
  service: Pick<Source, 'brand' | 'name' | 'logoUrl'>
  unavailable: boolean
  route?: boolean
  iconOnly?: boolean
}) {
  const named = !iconOnly && (!service.logoUrl || (!route && !WORDMARKS.has(service.brand ?? '')))
  return (
    <div
      data-part="node"
      data-testid="signal-node"
      data-unavailable={unavailable}
      className={`flex shrink-0 flex-col items-center gap-[0.15em] ${unavailable ? 'text-kiosk-text/50' : 'text-kiosk-text/80'}`}
    >
      <div className="flex items-center gap-[0.4em] font-semibold">
        {service.logoUrl ? (
          <Logo brand={service.brand} url={service.logoUrl} name={service.name} grey={unavailable} />
        ) : (
          <SourceIcon source={service} />
        )}
        {named && <span>{service.name}</span>}
      </div>
      {unavailable && (
        <>
          {' '}
          <span className="text-[0.7em] font-medium">unavailable</span>
        </>
      )}
    </div>
  )
}

/**
 * A calm wave between two ports. While the music plays, a pulse of light
 * runs along it towards the end and the arrival port lights up; only
 * transform and opacity animate. Its label goes when it is wider than the
 * wave.
 */
function Wave({ shade, moving, label }: { shade: 'main' | 'soft'; moving: boolean; label: WaveLabel | null }) {
  const wave = useRef<HTMLDivElement>(null)
  const text = useRef<HTMLSpanElement>(null)
  const [fits, setFits] = useState(true)

  useEffect(() => {
    const element = wave.current
    const labelText = text.current
    if (!element || !labelText || typeof ResizeObserver === 'undefined') return
    const fit = () => setFits(labelText.scrollWidth <= labelText.clientWidth + 1)
    const observer = new ResizeObserver(fit)
    observer.observe(element)
    return () => observer.disconnect()
    // A new label is measured afresh.
  }, [label?.main, label?.extra])

  const colour = { '--wave': shade === 'main' ? 'var(--signal)' : 'var(--signal-soft)' } as CSSProperties
  return (
    <div
      ref={wave}
      data-part="wave"
      data-testid="wave"
      data-shade={shade}
      style={colour}
      className="flex min-w-[4vmin] flex-1 basis-0 flex-col"
    >
      <span
        ref={text}
        data-testid="wave-label"
        className={`block overflow-hidden text-center text-[0.72em] font-semibold text-(--wave) tabular-nums opacity-70 ${fits ? '' : 'invisible'}`}
      >
        {label && (
          <>
            {label.main}
            {label.extra && (
              <>
                {' '}
                <span className="font-medium opacity-75">{label.extra}</span>
              </>
            )}
          </>
        )}
        {/* Keeps the label's line when there is no label. */}
        {!label && ' '}
      </span>
      <div className="relative h-[2.2vmin]">
        <span className="absolute top-1/2 left-0 size-[0.9vmin] -translate-y-1/2 rounded-full bg-(--wave) opacity-55" />
        {/* The glow: a blurred copy of the wave, still. */}
        <div className="absolute inset-y-0 right-[1.6vmin] left-[0.9vmin] opacity-45 blur-[0.35vmin]">
          <div className="signal-wave h-full bg-(--wave)" />
        </div>
        <div className="signal-wave absolute inset-y-0 right-[1.6vmin] left-[0.9vmin] overflow-hidden bg-linear-to-r from-(--wave)/25 to-(--wave)">
          {moving && (
            <span
              data-testid="pulse"
              className="absolute inset-y-0 left-0 w-1/5 -translate-x-full bg-linear-to-r from-transparent via-white to-transparent motion-safe:animate-signal-pulse"
            />
          )}
        </div>
        <span className="absolute top-1/2 right-0 size-[1.6vmin] -translate-y-1/2 rounded-full border-[0.2vmin] border-(--wave) bg-black/60 shadow-[0_0_0.9vmin_var(--wave)]">
          <span
            className={`absolute inset-[0.2vmin] rounded-full bg-(--wave) opacity-50 ${moving ? 'motion-safe:animate-signal-arrive' : ''}`}
          />
        </span>
      </div>
    </div>
  )
}

/**
 * How much of the row has given way: 0 nothing, 1 the sound box's last
 * line, 2 also the route's name. Measured when the row or the box changes
 * size; a new sound text, or a wider row, starts again from 0.
 */
function useGiveWay(
  row: RefObject<HTMLDivElement | null>,
  box: RefObject<HTMLDivElement | null>,
  text: string,
): number {
  const [fit, setFit] = useState({ text, level: 0 })

  useEffect(() => {
    const element = row.current
    if (!element || !box.current || typeof ResizeObserver === 'undefined') return
    let width = element.clientWidth
    let level = 0
    let frame = 0
    const measure = () => {
      if (element.clientWidth > width + 1) {
        level = 0
        setFit({ text, level })
      } else if (level < 2 && element.scrollWidth > element.clientWidth + 1) {
        // One step at a time; look again once it has taken effect.
        level += 1
        setFit({ text, level })
        frame = requestAnimationFrame(measure)
      }
      width = element.clientWidth
    }
    const observer = new ResizeObserver(measure)
    observer.observe(element)
    observer.observe(box.current)
    return () => {
      observer.disconnect()
      cancelAnimationFrame(frame)
    }
  }, [row, box, text])

  return fit.text === text ? fit.level : 0
}

/**
 * The sound the player receives: the Hi-Res mark for a Hi-Res path, the
 * format, the codec and what changed on the way. Its last line can be
 * shortened to the width of the first two.
 */
function SoundBox({
  signal,
  boxRef,
  squeezed,
  soft,
}: {
  signal: Signal
  boxRef: RefObject<HTMLDivElement | null>
  squeezed: boolean
  soft: boolean
}) {
  const [numbers, codec, changes] = soundText(signal)
  const border = soft ? 'var(--signal-soft)' : 'var(--signal)'
  return (
    <div
      ref={boxRef}
      data-part="sound"
      data-testid="sound-box"
      style={{ '--wave': border } as CSSProperties}
      className="flex shrink-0 items-center gap-[0.55em] rounded-[0.75em] bg-black/45 px-[0.7em] py-[0.3em] shadow-[inset_0_0_0_0.09em_var(--wave),0_0_1.1em_-0.35em_var(--wave)]"
    >
      {signal.class === 'hi-res' &&
        (signal.hiResLogoUrl ? (
          <img src={signal.hiResLogoUrl} alt="Hi-Res" className="block h-[2.5em] w-auto shrink-0 opacity-90" />
        ) : (
          <span className="shrink-0 rounded-full bg-linear-to-b from-[rgb(236_196_96)] to-[rgb(184_137_43)] px-[0.55em] py-[0.3em] text-[0.7em] font-bold tracking-[0.08em] text-[rgb(26_20_5)] [font-feature-settings:'case']">
            HI-RES
          </span>
        ))}
      <div className="flex flex-col leading-[1.12]">
        <span
          className={
            signal.class === 'unknown'
              ? 'text-[0.85em] font-medium text-kiosk-text/60'
              : 'text-[1.2em] font-semibold text-kiosk-text/85 tabular-nums'
          }
        >
          {numbers}
        </span>
        {codec && <span className="text-[0.85em] font-medium text-kiosk-text/60">{codec}</span>}
        {changes && (
          <span className={`truncate text-[0.75em] text-kiosk-text/45 ${squeezed ? 'w-0 min-w-full' : ''}`}>
            {changes}
          </span>
        )}
      </div>
    </div>
  )
}

interface WaveLabel {
  main: string
  extra: string | null
}

/** "24/96" and the codec, or "320 kbps". */
function sourceLabel(quality: Quality | null): WaveLabel | null {
  if (!quality) return null
  return { main: short(quality), extra: quality.bitrate ? null : quality.codec }
}

/** What the route sends, and the first thing it changed (or the codec). */
function deliveredLabel(signal: Signal): WaveLabel | null {
  const first = signal.changes[0]
  const change = first ? CHANGE_WORDS[first.kind] : null
  if (!signal.delivered) return change ? { main: change, extra: null } : null
  return { main: short(signal.delivered), extra: change ?? signal.delivered.codec }
}

const CHANGE_WORDS: Record<Change['kind'], string> = {
  resampled: 'resampled',
  bitDepth: 'bit depth',
  normalised: 'normalised',
  dsp: 'DSP',
}

/** The sound box's three lines; the codec and the changes may be empty. */
function soundText(signal: Signal): [string, string | null, string | null] {
  const quality = signal.delivered ?? signal.source
  if (signal.class === 'unknown' || !quality) return ['quality not reported', null, null]
  const codec = quality.codec ?? (quality.bitrate ? 'lossy' : null)
  const changes = signal.changes.map((change) => describe(change, signal)).join(' · ')
  return [numbers(quality), codec, changes || 'untouched']
}

function describe(change: Change, { source, delivered }: Signal): string {
  switch (change.kind) {
    case 'resampled':
      return source?.bitDepth && source.sampleRateHz
        ? `resampled from ${source.bitDepth} / ${kilohertz(source.sampleRateHz)}`
        : 'resampled'
    case 'bitDepth':
      return delivered?.bitDepth && source?.bitDepth
        ? `${delivered.bitDepth} bit from ${source.bitDepth}`
        : 'bit depth changed'
    case 'normalised':
      return change.gainDb === null ? 'normalised' : `normalised (${decibels(change.gainDb)})`
    case 'dsp':
      return 'DSP'
  }
}

/** "24 bit / 96 kHz", "320 kbps", or the label. */
function numbers(quality: Quality): string {
  if (quality.bitDepth && quality.sampleRateHz)
    return `${quality.bitDepth} bit / ${kilohertz(quality.sampleRateHz)} kHz`
  if (quality.bitrate) return `${Math.round(quality.bitrate / 1000)} kbps`
  return quality.label
}

/** "24/96", "320 kbps", or the label. */
function short(quality: Quality): string {
  if (quality.bitDepth && quality.sampleRateHz) return `${quality.bitDepth}/${kilohertz(quality.sampleRateHz)}`
  if (quality.bitrate) return `${Math.round(quality.bitrate / 1000)} kbps`
  return quality.label
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
