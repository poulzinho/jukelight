import { Airplay, AudioLines, Bluetooth, Cable, Folder, Laptop, Radio, Server, Speaker as SpeakerIcon } from 'lucide-react'
import { useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import { useFontsLoaded } from '../useFontsLoaded'
import type { Player } from '../api'
import { type Card, type Icon, type Ink, type Layout, type Measure, layout } from '../diagram/layout'
import { diagramModel } from '../diagram/model'

/** Diagram units per second for the pulse, and steps per second. */
const SPEED = 90
const STEPS_PER_SECOND = 15
/** The pulse's bright piece, in diagram units. */
const SPARK = 10
/** The mock's text size, in diagram units, that `text-signal` stands for. */
const BASE = 12.5

/**
 * How the music reaches every speaker: the source with what it streams,
 * Music Assistant, the speakers of the session, the player as a streamer and
 * its outputs, joined by lines. With a queue it runs down its column;
 * without one, across the width when it fits. It measures its box and lays
 * itself out to fit (see `layout`); one pulse runs along every line when
 * something changes.
 */
export function SignalDiagram({ player, queue }: { player: Player; queue: boolean }) {
  const box = useRef<HTMLDivElement>(null)
  const size = useBoxSize(box, queue)
  const aspect = useLogoAspect(player.source?.logoUrl ?? null)
  const model = useMemo(() => {
    const built = diagramModel(player)
    if (built && aspect) built.source.logoAspect = aspect
    return built
  }, [player, aspect])
  const measure = useMeasure(size.unit)
  if (!model) return null
  const laid = layout(model, size.width / size.unit, size.height / size.unit, queue, measure)
  const moving = player.state !== 'paused' && !model.source.unavailable
  // One pulse for each new thing to show: a new key remounts the sparks,
  // which runs their one-off animation once.
  const pulse = JSON.stringify([
    player.track?.title,
    player.track?.artists,
    player.source?.brand,
    player.source?.name,
    player.source?.via?.available,
    player.signal?.class,
    model.streamer.main,
    model.streamer.codec,
    player.speakers.map((speaker) => [speaker.name, speaker.playing, speaker.quality?.label]),
  ])
  return (
    <div ref={box} data-testid="signal-diagram" data-direction={laid.direction} className="min-h-0 min-w-0 text-signal">
      <svg
        width={laid.width * size.unit}
        height={laid.height * size.unit}
        viewBox={`0 0 ${laid.width} ${laid.height}`}
        role="img"
        aria-label={description(player)}
        className="block overflow-visible"
      >
        <Drawing laid={laid} />
        {moving && <Pulse key={pulse} laid={laid} />}
      </svg>
    </div>
  )
}

/** A sentence for assistive technology and tests: who plays, and how. */
function description(player: Player): string {
  const speakers = player.speakers.map((speaker) => speaker.name)
  const parts = [`${player.source?.name ?? player.session.label} to ${player.device.name}`]
  if (speakers.length) parts.push(`and ${speakers.join(', ')}`)
  return parts.join(' ')
}

function Drawing({ laid }: { laid: Layout }) {
  return (
    <>
      {laid.lines.map((line, i) => (
        <path key={`l${i}`} d={line.d} fill="none" stroke={ink(line.ink, line.soft)} strokeWidth={1} strokeOpacity={0.6} strokeLinecap="round" strokeLinejoin="round" />
      ))}
      {laid.dots.map((dot, i) => (
        <circle key={`d${i}`} cx={dot.x} cy={dot.y} r={1.9} fill={ink(dot.ink, dot.soft)} opacity={0.75} />
      ))}
      {laid.ports.map((port, i) => (
        <g key={`p${i}`}>
          <circle cx={port.x} cy={port.y} r={3.3} fill="#111318" stroke={ink(port.ink, port.soft)} strokeWidth={1.1} strokeOpacity={0.75} />
        </g>
      ))}
      {laid.pictures.map((picture, i) => (
        <image
          key={`i${i}`}
          data-part={picture.part}
          href={picture.url}
          x={picture.x}
          y={picture.y}
          width={picture.width}
          height={picture.height}
          preserveAspectRatio="xMinYMid meet"
          opacity={picture.grey ? 0.4 : 1}
          style={picture.grey ? { filter: 'grayscale(1)' } : undefined}
        >
          <title>{picture.alt}</title>
        </image>
      ))}
      {laid.icons.map((icon, i) => (
        <IconAt key={`c${i}`} icon={icon} />
      ))}
      {laid.texts.map((text, i) => (
        <g key={`t${i}`}>
          {text.pcm && <Pcm x={text.x + 4} y={text.y - 9.5} colour={ink(text.ink, text.soft)} />}
          <text
            data-part={text.part}
            x={text.x}
            y={text.y}
            fontSize={text.size}
            fontWeight={text.weight}
            fill={ink(text.ink, text.soft)}
            textAnchor={text.anchor}
            className="tabular-nums"
          >
            {text.text}
            {text.then && (
              <tspan dx={6} fontWeight={text.then.weight} data-part={text.then.part}>
                {text.then.text}
              </tspan>
            )}
          </text>
        </g>
      ))}
      <Streamer card={laid.card} />
    </>
  )
}

const CARD_HEIGHT = 58

/** The player as a streamer: a low card with a lid, a status light and feet. */
function Streamer({ card }: { card: Card }) {
  const { x, y, width: w } = card
  const colour = ink(card.ink, card.soft)
  const left = card.hiRes ? x + 51 : x + 12
  return (
    <g data-part="streamer">
      <rect x={x} y={y} width={w} height={CARD_HEIGHT} rx={7} fill="rgba(0,0,0,.5)" stroke={colour} strokeWidth={1.05} />
      <path
        d={`M${x + 7} ${y + 0.8} H${x + w - 7} Q${x + w - 0.8} ${y + 0.8} ${x + w - 0.8} ${y + 7} V${y + 19} H${x + 0.8} V${y + 7} Q${x + 0.8} ${y + 0.8} ${x + 7} ${y + 0.8} Z`}
        fill={colour}
        opacity={0.16}
      />
      <line x1={x + 1} y1={y + 19} x2={x + w - 1} y2={y + 19} stroke={colour} strokeWidth={1} opacity={0.4} />
      <circle cx={x + w - 11} cy={y + 9.8} r={2} fill={colour} />
      {/* The maker's mark, then the room, set like a speaker's name, in the lid. */}
      {card.markUrl && <image href={card.markUrl} x={x + 8} y={y + 3.6} width={12} height={12} opacity={0.85} />}
      <text data-part="streamer-room" x={card.markUrl ? x + 25 : x + 9} y={y + 14.2} fontSize={12.5} fontWeight={400} fill={ink('text', false)}>
        {card.room}
      </text>
      <rect x={x + 14} y={y + CARD_HEIGHT} width={18} height={3.5} rx={1.5} fill={colour} opacity={0.55} />
      <rect x={x + w - 32} y={y + CARD_HEIGHT} width={18} height={3.5} rx={1.5} fill={colour} opacity={0.55} />
      {card.hiRes &&
        (card.hiResLogoUrl ? (
          <image href={card.hiResLogoUrl} x={x + 10} y={y + 23} width={32} height={32}>
            <title>Hi-Res</title>
          </image>
        ) : (
          <g>
            <rect x={x + 8} y={y + 32} width={38} height={15} rx={7.5} fill="rgb(236 196 96)" />
            <text x={x + 27} y={y + 42.5} fontSize={8} fontWeight={700} fill="rgb(26 20 5)" textAnchor="middle" letterSpacing={0.6}>
              HI-RES
            </text>
          </g>
        ))}
      <text data-part="streamer-main" x={left} y={y + 36.5} fontSize={card.mainSize} fontWeight={650} fill="rgba(245,245,247,.92)" className="tabular-nums">
        {card.main}
      </text>
      {card.pcm && <Pcm x={left} y={y + 41.5} colour="rgba(245,245,247,.62)" />}
      {card.codec && (
        <text data-part="streamer-codec" x={card.pcm ? left + 17 : left} y={y + 50.5} fontSize={10.5} fontWeight={500} fill="rgba(245,245,247,.62)">
          {card.codec}
        </text>
      )}
    </g>
  )
}

/** Raw samples: a stepped wave. */
function Pcm({ x, y, colour }: { x: number; y: number; colour: string }) {
  return (
    <path
      data-part="pcm"
      transform={`translate(${x} ${y}) scale(0.72)`}
      d="M0 6 H2.5 V2 H5 V0 H7.5 V3 H10 V8 H12.5 V11 H15 V9 H17.5 V5"
      fill="none"
      stroke={colour}
      strokeWidth={1.6}
      strokeLinejoin="round"
      strokeLinecap="round"
    />
  )
}

function IconAt({ icon }: { icon: Icon }) {
  const colour = icon.ink === 'text' ? 'rgba(245,245,247,.85)' : ink(icon.ink, icon.soft)
  const props = { x: icon.x - 9, y: icon.y - 9, width: 18, height: 18, color: colour, strokeWidth: 1.4, 'aria-hidden': true } as const
  switch (icon.kind) {
    case 'speaker':
      return <SpeakerIcon data-part="icon-speaker" {...props} />
    case 'computer':
      return <Laptop data-part="icon-computer" {...props} />
    case 'subwoofer':
      return (
        <g data-part="icon-subwoofer" transform={`translate(${icon.x - 8.5} ${icon.y - 8.5}) scale(0.95)`} fill="none" stroke={colour} strokeWidth={1.1}>
          <rect x={1} y={1} width={16} height={16} rx={3} />
          <circle cx={9} cy={9} r={4.4} />
          <circle cx={9} cy={9} r={1.4} />
        </g>
      )
    case 'airplay':
      return <Airplay {...props} />
    case 'bluetooth':
      return <Bluetooth {...props} />
    case 'radio':
      return <Radio {...props} />
    case 'server':
      return <Server {...props} />
    case 'folder':
      return <Folder {...props} />
    case 'input':
      return <Cable {...props} />
    case 'audio':
      return <AudioLines {...props} />
  }
}

/**
 * One pulse along the whole tree: a bright piece moves along each straight
 * segment, starting when the pulse reaches it, so every branch leaving a
 * junction starts at the same moment; each port lights up as it arrives.
 * Only transform and opacity animate, stepped at about 15 a second; with
 * reduced motion nothing runs (see `index.css`).
 */
function Pulse({ laid }: { laid: Layout }) {
  // The source's wave and its format light up at once as the pulse starts.
  const wave = laid.lines.find((line) => line.part === 'wave')
  const formats = laid.texts.filter((text) => text.part === 'streams' || text.part === 'streams-codec')
  return (
    <g data-testid="pulse">
      {wave && (
        <g data-part="wave-flash" className="signal-flash" style={{ animationDelay: '0ms' }}>
          <path d={wave.d} fill="none" stroke="#fff" strokeWidth={1} strokeLinecap="round" />
          {formats.map((text, i) => (
            // Set exactly like the format, so the copy lies over it.
            <text key={i} x={text.x} y={text.y} fontSize={text.size} fontWeight={text.weight} textAnchor={text.anchor} fill="#fff" className="tabular-nums">
              {text.text}
            </text>
          ))}
        </g>
      )}
      {laid.lines.flatMap((line, i) =>
        line.segments.map((segment, j) => {
          const length = Math.hypot(segment.x2 - segment.x1, segment.y2 - segment.y1)
          if (length < 1) return null
          const run = Math.max(0, length - SPARK)
          const ux = (segment.x2 - segment.x1) / length
          const uy = (segment.y2 - segment.y1) / length
          const seconds = Math.max(run / SPEED, 0.1)
          const style = {
            '--dx': `${ux * run}px`,
            '--dy': `${uy * run}px`,
            animationDelay: `${Math.round((segment.from / SPEED) * 1000)}ms`,
            animationDuration: `${Math.round(seconds * 1000)}ms`,
            animationTimingFunction: `steps(${Math.max(1, Math.round(seconds * STEPS_PER_SECOND))})`,
          } as CSSProperties
          return (
            <line
              key={`s${i}-${j}`}
              data-part="spark"
              className="signal-spark"
              x1={segment.x1}
              y1={segment.y1}
              x2={segment.x1 + ux * SPARK}
              y2={segment.y1 + uy * SPARK}
              stroke="#fff"
              strokeWidth={2}
              strokeLinecap="round"
              style={style}
            />
          )
        }),
      )}
      {laid.ports.map((port, i) => (
        <circle
          key={`a${i}`}
          data-part="arrive"
          className="signal-arrive"
          cx={port.x}
          cy={port.y}
          r={2.2}
          fill={ink(port.ink, port.soft)}
          style={{ animationDelay: `${Math.round((port.arrive / SPEED) * 1000)}ms` }}
        />
      ))}
    </g>
  )
}

/** The colour for a class, or the faint grey of what doesn't play. */
function ink(value: Ink | 'text' | 'dim', soft: boolean): string {
  switch (value) {
    case 'text':
      return 'rgba(245,245,247,.85)'
    case 'dim':
      return 'rgba(245,245,247,.5)'
    case 'faint':
      return 'rgba(245,245,247,.35)'
    default:
      return `var(--color-signal-${value}${soft ? '-soft' : ''})`
  }
}

interface BoxSize {
  width: number
  height: number
  /** Screen pixels per diagram unit. */
  unit: number
}

/**
 * The room the diagram has: its box's width, and the height down to the
 * footer; the unit follows `text-signal`, so it scales with the screen.
 * Measured whenever the box changes; generous defaults until then (tests).
 */
function useBoxSize(ref: React.RefObject<HTMLDivElement | null>, queue: boolean): BoxSize {
  // Until measured (and in tests): a column beside the queue, or the full width.
  const [size, setSize] = useState<BoxSize>({ width: queue ? 236 : 524, height: 10_000, unit: 1 })
  // Measured before the first paint, so a screen that comes back (after the
  // idle screen) shows the diagram at its size at once, without a jump.
  useLayoutEffect(() => {
    const element = ref.current
    if (!element || typeof ResizeObserver === 'undefined') return
    const update = () => {
      const unit = parseFloat(getComputedStyle(element).fontSize) / BASE || 1
      // Down to the page's bottom margin: in portrait the area below the
      // progress ends there; in landscape, the same margin from the screen's
      // edge (8vmin).
      const vmin = Math.min(window.innerWidth, window.innerHeight) / 100
      const area = element.parentElement?.getBoundingClientRect()
      const portrait = window.innerHeight > window.innerWidth
      const bottom = portrait && area ? area.bottom : window.innerHeight - 8 * vmin
      const top = element.getBoundingClientRect().top
      const next = { width: element.clientWidth, height: Math.max(0, bottom - top), unit }
      setSize((current) =>
        Math.abs(current.width - next.width) < 1 && Math.abs(current.height - next.height) < 1 && current.unit === next.unit
          ? current
          : next,
      )
    }
    update()
    const observer = new ResizeObserver(update)
    observer.observe(element)
    observer.observe(document.documentElement)
    return () => observer.disconnect()
  }, [ref])
  return size
}

/** A canvas to measure text with the page's font; `null` where there is none (tests). */
let canvas: CanvasRenderingContext2D | null | undefined

/** Widths measured so far, for one generation of loaded fonts. */
const widths = new Map<string, number>()
let widthsGeneration = 0

/** The width of `text` in `font`; `generation` changes when fonts finish loading. */
function textWidth(text: string, font: string, generation: number): number | null {
  if (canvas === undefined) {
    canvas = typeof navigator !== 'undefined' && /jsdom/.test(navigator.userAgent) ? null : document.createElement('canvas').getContext('2d')
  }
  if (!canvas) return null
  if (generation !== widthsGeneration) {
    widths.clear()
    widthsGeneration = generation
  }
  const key = `${font}|${text}`
  const known = widths.get(key)
  if (known !== undefined) return known
  canvas.font = font
  const width = canvas.measureText(text).width
  widths.set(key, width)
  return width
}

/**
 * Measures text with the page's font, in diagram units; an estimate without
 * a canvas. Measured again once the page's fonts have loaded.
 */
function useMeasure(unit: number): Measure {
  const fontsLoaded = useFontsLoaded()
  return useMemo(() => {
    const family = getComputedStyle(document.body).fontFamily || 'sans-serif'
    return (text, size, bold) => {
      const width = textWidth(text, `${bold ? 650 : 400} ${size * unit}px ${family}`, fontsLoaded)
      // Without a canvas (tests): Figtree's average widths.
      return width === null ? text.length * size * (bold ? 0.5 : 0.47) : width / unit
    }
  }, [unit, fontsLoaded])
}

/** Logos' proportions, kept across screens: a logo that comes back keeps its size. */
const aspects = new Map<string, number>()

function useLogoAspect(url: string | null): number | undefined {
  const [aspect, setAspect] = useState<{ url: string; value: number } | null>(null)
  useEffect(() => {
    if (!url || aspects.has(url)) return
    const image = new Image()
    image.onload = () => {
      if (!image.naturalWidth || !image.naturalHeight) return
      const value = image.naturalWidth / image.naturalHeight
      aspects.set(url, value)
      setAspect({ url, value })
    }
    image.src = url
  }, [url])
  if (!url) return undefined
  return aspects.get(url) ?? (aspect && aspect.url === url ? aspect.value : undefined)
}
