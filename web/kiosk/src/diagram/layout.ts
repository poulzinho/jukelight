import type { QualityClass } from '../api'
import type { DiagramModel, OutputNode, SpeakerRow } from './model'

/**
 * Lays out the signal diagram: everything placed in "diagram units" (the
 * component scales them to the screen), lines with right angles and rounded
 * corners, and every text fitted to the room it has. The same coordinates as
 * the agreed mocks (`/tmp/speakers/gen2.py` during DEV-45).
 */

/** Measures a text's width in diagram units. */
export type Measure = (text: string, size: number, bold: boolean) => number

export type Ink = QualityClass | 'faint'

export interface Line {
  d: string
  ink: Ink
  soft: boolean
  /** Its straight pieces, for the pulse, with how far along the tree each starts. */
  segments: Segment[]
  /** What the line stands for, when it isn't a path the pulse runs along. */
  part?: 'wave'
}

export interface Segment {
  x1: number
  y1: number
  x2: number
  y2: number
  /** Distance from the source to the segment's start. */
  from: number
}

export interface Port {
  x: number
  y: number
  ink: Ink
  soft: boolean
  /** Distance from the source, when the pulse arrives. */
  arrive: number
}

export interface Dot {
  x: number
  y: number
  ink: Ink
  soft: boolean
}

export type IconKind = 'speaker' | 'computer' | 'subwoofer' | 'airplay' | 'bluetooth' | 'radio' | 'server' | 'folder' | 'input' | 'audio'

export interface Icon {
  kind: IconKind
  x: number
  y: number
  ink: Ink | 'text'
  soft: boolean
}

export interface Text {
  x: number
  y: number
  text: string
  size: number
  weight: number
  ink: Ink | 'text' | 'dim'
  soft: boolean
  anchor: 'start' | 'end'
  /** A PCM icon before the text. */
  pcm?: boolean
  part?: string
  /** More on the same line, set by the browser after a space: "FLAC" after the format. */
  then?: { text: string; weight: number; part: string }
}

export interface Picture {
  x: number
  y: number
  width: number
  height: number
  url: string
  alt: string
  grey: boolean
  part: string
}

export interface Card {
  x: number
  y: number
  width: number
  ink: Ink
  soft: boolean
  main: string
  mainSize: number
  codec: string
  room: string
  pcm: boolean
  hiRes: boolean
  hiResLogoUrl: string | null
  markUrl: string | null
}

export interface Layout {
  width: number
  height: number
  direction: 'down' | 'across'
  lines: Line[]
  ports: Port[]
  dots: Dot[]
  icons: Icon[]
  texts: Text[]
  pictures: Picture[]
  card: Card
}

const R = 7
const ROW = 32
const MORE = 24
const GAP = 26
const SEG = 34
const SEG_COMPACT = 40
const CARD_HEIGHT = 58
const CARD_WIDTH_ACROSS = 176
const OUTPUT_ROW = 26

/** Shortens `text` with "…" to fit `room`. */
export function fit(text: string, size: number, room: number, bold: boolean, measure: Measure): string {
  if (measure(text, size, bold) <= room) return text
  for (let keep = text.length - 1; keep > 0; keep--) {
    const candidate = `${text.slice(0, keep).trimEnd()}…`
    if (measure(candidate, size, bold) <= room) return candidate
  }
  return '…'
}

interface Options {
  shown: number
  compact: boolean
  inline: boolean
  outputs: boolean
}

/**
 * With a queue the diagram runs down a column of `width`; without one it
 * runs across, if every part fits in full, otherwise down. Within `height`
 * it gives way in a fixed order: speakers fold into "+ N more", the source's
 * line gets shorter, the outputs go onto one line as icons, and last the
 * outputs are left out.
 */
export function layout(model: DiagramModel, width: number, height: number, queue: boolean, measure: Measure): Layout {
  if (!queue) {
    const across = layoutAcross(model, width, measure)
    if (across) return across
  }
  const column = queue ? width : Math.min(width, 300)
  const options: Options[] = []
  for (let shown = model.speakers.length; shown >= 0; shown--) options.push({ shown, compact: false, inline: false, outputs: true })
  options.push({ shown: 0, compact: true, inline: false, outputs: true })
  options.push({ shown: 0, compact: true, inline: true, outputs: true })
  options.push({ shown: 0, compact: true, inline: true, outputs: false })
  let last: Layout | null = null
  for (const option of options) {
    last = layoutDown(model, column, option, measure)
    if (last.height <= height) return last
  }
  return last as Layout
}

function source(model: DiagramModel, x: number, cy: number, maxWidth: number, measure: Measure, out: Layout): number {
  const { source: node } = model
  if (node.logoUrl) {
    // Wordmarks 18 high, square marks 24; a wide wordmark shrinks to maxWidth.
    const ratio = node.logoAspect ?? 3.2
    let h = ratio > 2 ? 18 : 24
    let w = Math.min(h * ratio, maxWidth)
    h = w / ratio
    out.pictures.push({ x, y: cy - h / 2, width: w, height: h, url: node.logoUrl, alt: node.name, grey: node.unavailable, part: 'source' })
    if (node.unavailable) {
      out.texts.push({ x: x + w + 6, y: cy + 4.5, text: 'unavailable', size: 11.5, weight: 500, ink: 'dim', soft: false, anchor: 'start', part: 'source' })
      w += 6 + measure('unavailable', 11.5, false)
    }
    return w
  }
  const label = node.unavailable ? 'unavailable' : fit(node.name, 12.5, maxWidth - 26, true, measure)
  out.icons.push({ kind: sourceIcon(node.brand, node.name), x: x + 10, y: cy, ink: node.unavailable ? 'faint' : 'text', soft: false })
  out.texts.push({ x: x + 26, y: cy + 4.5, text: label, size: 12.5, weight: node.unavailable ? 500 : 650, ink: node.unavailable ? 'dim' : 'text', soft: false, anchor: 'start', part: 'source' })
  return Math.min(maxWidth, 26 + measure(label, 12.5, true))
}

function sourceIcon(brand: string | null, name: string): IconKind {
  switch (brand) {
    case 'airplay':
      return 'airplay'
    case 'bluetooth':
      return 'bluetooth'
    case 'subsonic':
      return 'server'
    case 'library':
      return 'folder'
  }
  if (/input|HDMI ARC/i.test(name)) return 'input'
  if (name === 'Stream' || !brand) return 'radio'
  return 'audio'
}

function routeNode(model: DiagramModel, x: number, y: number, out: Layout) {
  const route = model.route
  if (!route) return
  if (route.logoUrl) out.pictures.push({ x, y, width: 24, height: 24, url: route.logoUrl, alt: route.name, grey: false, part: 'route' })
  else out.icons.push({ kind: 'server', x: x + 12, y: y + 12, ink: 'text', soft: false })
}

/** The shortest wave between the source's logo and its format. */
const WAVE_MIN = 40

/** Where the format of what the source streams starts, right-aligned at `right`. */
function formatLeft(model: DiagramModel, right: number, measure: Measure): number {
  const s = model.streams
  return s ? right - measure(s.codec, 11, false) - 7 - measure(s.short, 11, true) : right
}

/**
 * What the source streams: the format and codec ("24/44.1  FLAC") at
 * `right`, set like the speakers' formats, and, from `fromX`, a gentle wave
 * out to it on the same line. The wave is no path for the pulse.
 */
function streams(model: DiagramModel, fromX: number | null, right: number, y: number, measure: Measure, out: Layout) {
  const s = model.streams
  if (!s) return
  const ink = model.sourceClass
  const codecW = measure(s.codec, 11, false)
  out.texts.push({ x: right, y, text: s.codec, size: 11, weight: 500, ink, soft: true, anchor: 'end', part: 'streams-codec' })
  out.texts.push({ x: right - codecW - 7, y, text: s.short, size: 11, weight: 650, ink, soft: true, anchor: 'end', part: 'streams' })
  if (fromX === null) return
  const ly = y - 4
  const end = formatLeft(model, right, measure) - 8
  const half = 9
  const amplitude = 1.8
  let d = `M${fromX},${ly}`
  let x = fromX
  let up = true
  while (x + half <= end) {
    d += ` q${half / 2},${up ? -amplitude * 2 : amplitude * 2} ${half},0`
    x += half
    up = !up
  }
  out.lines.push({ d, ink, soft: true, segments: [], part: 'wave' })
}

function emptyLayout(direction: Layout['direction']): Layout {
  return { width: 0, height: 0, direction, lines: [], ports: [], dots: [], icons: [], texts: [], pictures: [], card: null as unknown as Card }
}

function card(model: DiagramModel, x: number, y: number, width: number, measure: Measure): Card {
  const s = model.streamer
  const left = s.hiRes ? 51 : 12
  const room = width - 10 - left
  const mainSize = measure(s.main, 13, true) <= room ? 13 : 12
  return {
    x,
    y,
    width,
    ink: model.playerClass,
    soft: model.soft,
    main: s.main,
    mainSize,
    codec: fit(s.codec, 10.5, room - (s.codec.startsWith('PCM') ? 17 : 0), false, measure),
    // In the lid, set like a speaker's name, after the mark.
    room: fit(s.room, 12.5, width - 30 - (s.markUrl ? 16 : 0), false, measure),
    pcm: s.codec.startsWith('PCM'),
    hiRes: s.hiRes,
    hiResLogoUrl: s.hiResLogoUrl,
    markUrl: s.markUrl,
  }
}

function speakerIcon(speaker: SpeakerRow): IconKind {
  return speaker.kind === 'computer' ? 'computer' : 'speaker'
}

function outputIcon(output: OutputNode): IconKind {
  return output.kind === 'subwoofer' ? 'subwoofer' : 'speaker'
}

/** Width of a speaker's quality, right-aligned: "16/44.1 [PCM] PCM". */
function qualityWidth(speaker: SpeakerRow, measure: Measure): number {
  if (!speaker.quality) return 0
  return measure(speaker.quality, 11, true) + 7 + (speaker.codec === 'PCM' ? 17 : 0) + measure(speaker.codec, 11, false)
}

function layoutDown(model: DiagramModel, w: number, option: Options, measure: Measure): Layout {
  const out = emptyLayout('down')
  // The source's logo starts at the left edge, level with the progress bar.
  const tx = 12
  const px = 36
  const ix = 52
  const nx = 66
  const trunkInk = model.playerClass
  const soft = model.soft
  const seg = option.compact ? SEG_COMPACT : SEG

  // A logo leaves room for at least a short wave to what it streams.
  const logoRoom = model.streams ? formatLeft(model, w - 2, measure) - 8 - (WAVE_MIN + 9) - 8 - (tx - 12) : w - 2
  const sourceW = source(model, tx - 12, 9, logoRoom, measure, out)
  const sourceInk: Ink = model.streams ? model.sourceClass : 'faint'
  out.lines.push({ d: `M${tx},22 V${seg}`, ink: sourceInk, soft: false, segments: [{ x1: tx, y1: 22, x2: tx, y2: seg, from: 0 }] })
  streams(model, tx - 12 + sourceW + 8, w - 2, 9 + 4, measure, out)
  let y = seg
  let along = seg - 22
  if (model.route) {
    routeNode(model, tx - 12, seg, out)
    y = seg + 24
    along += 24
  }

  const shown = model.speakers.slice(0, option.shown)
  const hidden = model.speakers.length - shown.length
  let cur = y + ROW - 8
  const rows: { kind: 'speaker' | 'more'; speaker?: SpeakerRow; y: number }[] = []
  for (const speaker of shown) {
    rows.push({ kind: 'speaker', speaker, y: cur })
    cur += ROW
  }
  if (hidden) {
    rows.push({ kind: 'more', y: cur - ROW + MORE })
    cur += MORE
  }
  const ny = rows.length ? cur - ROW + 4 + GAP + 28 - 6 : y + GAP + 22
  const top = ny - 29

  // The trunk down to the streamer, turning into its port.
  // Without Music Assistant, the source's line runs on to the streamer in one colour.
  const lineInk = model.route ? trunkInk : sourceInk
  out.lines.push({
    d: `M${tx},${y} V${ny - R} Q${tx},${ny} ${tx + R},${ny} H${px - 5}`,
    ink: lineInk,
    soft,
    segments: [
      { x1: tx, y1: y, x2: tx, y2: ny, from: along },
      { x1: tx, y1: ny, x2: px - 5, y2: ny, from: along + (ny - y) },
    ],
  })
  out.ports.push({ x: px, y: ny, ink: lineInk, soft, arrive: along + (ny - y) + (px - tx) })

  for (const row of rows) {
    const reach = along + (row.y - y)
    if (row.kind === 'more') {
      out.lines.push({ d: `M${tx},${row.y} H${px - 5}`, ink: trunkInk, soft, segments: [{ x1: tx, y1: row.y, x2: px - 5, y2: row.y, from: reach }] })
      out.dots.push({ x: tx, y: row.y, ink: trunkInk, soft })
      out.ports.push({ x: px, y: row.y, ink: trunkInk, soft, arrive: reach + px - tx })
      out.texts.push({ x: ix - 8, y: row.y + 4, text: `+ ${hidden} more`, size: 12, weight: 400, ink: 'dim', soft: false, anchor: 'start', part: 'more' })
      continue
    }
    const speaker = row.speaker as SpeakerRow
    const ink: Ink = speaker.playing ? speaker.class : 'faint'
    out.lines.push({ d: `M${tx},${row.y} H${px - 5}`, ink, soft: true, segments: [{ x1: tx, y1: row.y, x2: px - 5, y2: row.y, from: reach }] })
    out.dots.push({ x: tx, y: row.y, ink, soft: true })
    out.ports.push({ x: px, y: row.y, ink, soft: true, arrive: reach + px - tx })
    out.icons.push({ kind: speakerIcon(speaker), x: ix, y: row.y, ink, soft: true })
    let qualityRoom: number
    if (!speaker.playing) {
      out.texts.push({ x: w - 2, y: row.y + 4, text: 'not playing', size: 10.5, weight: 400, ink: 'dim', soft: false, anchor: 'end', part: 'speaker-quality' })
      qualityRoom = measure('not playing', 10.5, false)
    } else if (speaker.quality) {
      out.texts.push({ x: w - 2, y: row.y + 4, text: speaker.codec, size: 11, weight: 500, ink, soft: true, anchor: 'end', part: 'speaker-quality' })
      const codecW = measure(speaker.codec, 11, false) + 7
      out.texts.push({
        x: w - 2 - codecW - (speaker.codec === 'PCM' ? 17 : 0),
        y: row.y + 4,
        text: speaker.quality,
        size: 11,
        weight: 650,
        ink,
        soft: true,
        anchor: 'end',
        pcm: speaker.codec === 'PCM',
        part: 'speaker-quality',
      })
      qualityRoom = qualityWidth(speaker, measure)
    } else {
      qualityRoom = 0
    }
    out.texts.push({
      x: nx,
      y: row.y + 4,
      text: fit(speaker.name, 12.5, w - 2 - qualityRoom - 10 - nx, false, measure),
      size: 12.5,
      weight: 400,
      ink: speaker.playing ? 'text' : 'dim',
      soft: false,
      anchor: 'start',
      part: 'speaker-name',
    })
  }

  const cardWidth = Math.min(w - px - 6, 220)
  out.card = card(model, px + 6, top, cardWidth, measure)
  let h = top + CARD_HEIGHT + 4
  const atCard = along + (ny - y) + (px - tx)
  const outputs = option.outputs ? model.outputs : []
  if (outputs.length && option.inline) {
    const bx = px + 30
    const oy = top + CARD_HEIGHT + 22
    const xs = outputs.map((_, k) => bx + 26 + k * 40)
    const last = xs[xs.length - 1] as number
    out.lines.push({
      d: `M${bx},${top + 61.5} V${oy - R} Q${bx},${oy} ${bx + R},${oy} H${last - 5}`,
      ink: trunkInk,
      soft,
      segments: [
        { x1: bx, y1: top + 61.5, x2: bx, y2: oy, from: atCard },
        { x1: bx, y1: oy, x2: last - 5, y2: oy, from: atCard + (oy - top - 59.5) },
      ],
    })
    outputs.forEach((output, k) => {
      const x = xs[k] as number
      out.ports.push({ x, y: oy, ink: trunkInk, soft, arrive: atCard + (oy - top - 59.5) + (x - bx) })
      out.icons.push({ kind: outputIcon(output), x: x + 16, y: oy, ink: trunkInk, soft })
    })
    h = oy + 12
  } else if (outputs.length) {
    const bx = px + 30
    const opx = bx + 26
    const oys = outputs.map((_, k) => top + CARD_HEIGHT + 22 + k * OUTPUT_ROW)
    const lastY = oys[oys.length - 1] as number
    out.lines.push({
      d: `M${bx},${top + 61.5} V${lastY - R} Q${bx},${lastY} ${bx + R},${lastY} H${opx - 5}`,
      ink: trunkInk,
      soft,
      segments: [
        { x1: bx, y1: top + 61.5, x2: bx, y2: lastY, from: atCard },
        { x1: bx, y1: lastY, x2: opx - 5, y2: lastY, from: atCard + (lastY - top - 59.5) },
      ],
    })
    outputs.forEach((output, k) => {
      const oy = oys[k] as number
      const reach = atCard + (oy - top - 59.5)
      if (k < outputs.length - 1) {
        out.lines.push({ d: `M${bx},${oy} H${opx - 5}`, ink: trunkInk, soft, segments: [{ x1: bx, y1: oy, x2: opx - 5, y2: oy, from: reach }] })
        out.dots.push({ x: bx, y: oy, ink: trunkInk, soft })
      }
      out.ports.push({ x: opx, y: oy, ink: trunkInk, soft, arrive: reach + (opx - bx) })
      out.icons.push({ kind: outputIcon(output), x: opx + 16, y: oy, ink: trunkInk, soft })
      out.texts.push({ x: opx + 31, y: oy + 4, text: fit(output.name, 12, w - opx - 33, false, measure), size: 12, weight: 400, ink: 'text', soft: false, anchor: 'start', part: 'output' })
    })
    h = lastY + 10
  }
  out.width = w
  out.height = h
  return out
}

/**
 * Across the width: the source on the left with what it streams under it, a
 * line to the streamer, the outputs to its right. `null` when it doesn't fit
 * in full, the outputs' names included, or when other speakers play along.
 */
/**
 * The streamer's width when the diagram runs across: as wide as its lines
 * need, the mark included, between 140 and CARD_WIDTH_ACROSS.
 */
function acrossCardWidth(model: DiagramModel, measure: Measure): number {
  const s = model.streamer
  const pcm = s.codec.startsWith('PCM') ? 17 : 0
  const content = Math.max(
    measure(s.main, 13, true) + (s.markUrl ? 26 : 0),
    measure(s.codec, 10.5, false) + pcm,
    measure(s.room, 9.5, false),
  )
  return Math.min(CARD_WIDTH_ACROSS, Math.max(140, (s.hiRes ? 51 : 12) + content + 12))
}

function layoutAcross(model: DiagramModel, w: number, measure: Measure): Layout | null {
  if (model.speakers.length) return null
  const out = emptyLayout('across')
  const cy = 40
  const trunkInk = model.playerClass
  const soft = model.soft
  const mw = source(model, 0, cy, 110, measure, out)
  const labelW = model.streams ? measure(model.streams.short, 11, true) + measure(model.streams.codec, 11, false) + 7 : 0
  const outW = model.outputs.length ? Math.max(...model.outputs.map((o) => measure(o.name, 12, false))) + 79 : 0
  const routeW = model.route ? 44 : 0
  const cardW = acrossCardWidth(model, measure)
  const line = w - mw - routeW - cardW - outW - 10
  if (line < 34 || labelW > mw + 10 + routeW + line - 12) return null

  if (model.streams) streams(model, null, mw + 10 + routeW + line - 6, cy + 24, measure, out)
  let x0 = mw + 10
  let along = 0
  if (model.route) {
    out.lines.push({ d: `M${x0},${cy} H${x0 + 16}`, ink: model.sourceClass, soft: false, segments: [{ x1: x0, y1: cy, x2: x0 + 16, y2: cy, from: 0 }] })
    routeNode(model, x0 + 18, cy - 12, out)
    x0 += routeW
    along = routeW
  }
  const sx = x0 + line
  out.lines.push({
    d: `M${x0},${cy} H${sx - 5}`,
    ink: model.streams ? trunkInk : 'faint',
    soft,
    segments: [{ x1: x0, y1: cy, x2: sx - 5, y2: cy, from: along }],
  })
  out.ports.push({ x: sx, y: cy, ink: model.streams ? trunkInk : 'faint', soft, arrive: along + line })
  out.card = card(model, sx + 6, cy - 28, cardW, measure)
  const atCard = along + line
  if (model.outputs.length) {
    const ex = sx + 6 + cardW
    const bx = ex + 20
    const opx = bx + 24
    out.lines.push({ d: `M${ex},${cy} H${bx}`, ink: trunkInk, soft, segments: [{ x1: ex, y1: cy, x2: bx, y2: cy, from: atCard }] })
    out.dots.push({ x: bx, y: cy, ink: trunkInk, soft })
    model.outputs.forEach((output, k) => {
      const oy = cy - 14 + k * 28
      const turn = oy < cy ? oy + R : oy - R
      out.lines.push({
        d: `M${bx},${cy} V${turn} Q${bx},${oy} ${bx + R},${oy} H${opx - 5}`,
        ink: trunkInk,
        soft,
        segments: [
          { x1: bx, y1: cy, x2: bx, y2: oy, from: atCard + 20 },
          { x1: bx, y1: oy, x2: opx - 5, y2: oy, from: atCard + 20 + Math.abs(oy - cy) },
        ],
      })
      out.ports.push({ x: opx, y: oy, ink: trunkInk, soft, arrive: atCard + 20 + Math.abs(oy - cy) + 24 })
      out.icons.push({ kind: outputIcon(output), x: opx + 16, y: oy, ink: trunkInk, soft })
      out.texts.push({ x: opx + 31, y: oy + 4, text: output.name, size: 12, weight: 400, ink: 'text', soft: false, anchor: 'start', part: 'output' })
    })
  }
  out.width = w
  out.height = 96
  return out
}
