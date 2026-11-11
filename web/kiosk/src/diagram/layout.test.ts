// Spec: kiosk-display, "Signal diagram" and "Diagram fits": the layout,
// with a simple character-based measure.
import { expect, test } from 'vitest'
import type { Player } from '../api'
import { player, quality } from '../test/fixtures'
import { type Layout, type Measure, type Port, fit, layout } from './layout'
import { diagramModel } from './model'

// Roughly Inter's average widths.
const measure: Measure = (text, size, bold) => text.length * size * (bold ? 0.56 : 0.5)

const qobuzViaMusicAssistant = {
  brand: 'qobuz',
  name: 'Qobuz',
  logoUrl: null,
  via: { brand: 'music-assistant', name: 'Music Assistant', logoUrl: '/api/v1/logos/music-assistant?v=1', available: true },
}

const speaker = (name: string): Player['speakers'][number] => ({
  name,
  kind: 'speaker',
  playing: true,
  quality: { ...quality('PCM 16/44.1', null, false), class: 'lossless' },
})

function group(count: number, overrides: Partial<Player> = {}): Player {
  return player({
    source: qobuzViaMusicAssistant,
    speakers: Array.from({ length: count }, (_, i) => speaker(`Speaker ${i + 1}`)),
    outputs: [
      { name: 'Example Speaker', kind: 'speaker' },
      { name: 'Subwoofer', kind: 'subwoofer' },
    ],
    ...overrides,
  })
}

function laidOut(snapshot: Player, width = 236, height = 262, queue = true): Layout {
  const model = diagramModel(snapshot)
  if (!model) throw new Error('no model')
  return layout(model, width, height, queue, measure)
}

const texts = (out: Layout, part: string) => out.texts.filter((t) => t.part === part).map((t) => t.text)

test('a group with a queue runs down: two speaker rows, the streamer, its two outputs', () => {
  const out = laidOut(group(2))
  expect(out.direction).toBe('down')
  expect(texts(out, 'speaker-name')).toEqual(['Speaker 1', 'Speaker 2'])
  expect(texts(out, 'output')).toEqual(['Example Speaker', 'Subwoofer'])
  expect(out.height).toBeLessThanOrEqual(262)
})

test('without a queue and other speakers it runs across when it fits', () => {
  const out = laidOut(player({ outputs: [{ name: 'Subwoofer', kind: 'subwoofer' }] }), 524, 262, false)
  expect(out.direction).toBe('across')
})

test('without a queue but too narrow, it runs down', () => {
  const out = laidOut(player({ outputs: [{ name: 'A Very Long Output Speaker Name Indeed', kind: 'speaker' }] }), 300, 262, false)
  expect(out.direction).toBe('down')
})

test('many speakers fold into "+ N more", and the diagram fits', () => {
  const out = laidOut(group(5))
  expect(out.height).toBeLessThanOrEqual(262)
  expect(texts(out, 'more')).toHaveLength(1)
  const shown = texts(out, 'speaker-name').length
  expect(texts(out, 'more')[0]).toBe(`+ ${5 - shown} more`)
})

test('when even one row is too many, the source line shortens, then the outputs go onto one line, then away', () => {
  const tight = laidOut(group(2), 236, 170)
  expect(tight.height).toBeLessThanOrEqual(170)
  expect(texts(tight, 'speaker-name')).toEqual([])
  expect(texts(tight, 'streams')).not.toContain('streams')
})

test('long names are shortened to their room, never over the quality', () => {
  const out = laidOut(group(1, { speakers: [{ ...speaker('Upstairs Bathroom Speaker With A Long Name') }] }))
  const [name] = texts(out, 'speaker-name')
  expect(name).toMatch(/…$/)
  const nameText = out.texts.find((t) => t.part === 'speaker-name')
  const quality = out.texts.filter((t) => t.part === 'speaker-quality')
  const qualityLeft = Math.min(...quality.map((t) => t.x - measure(t.text, t.size, t.weight > 500)))
  expect((nameText?.x ?? 0) + measure(name ?? '', 12.5, false)).toBeLessThanOrEqual(qualityLeft)
})

test('a long output name is shortened with "…", never past the diagram', () => {
  const out = laidOut(group(1, { outputs: [{ name: 'Example Speaker With A Very Long Name', kind: 'speaker' }] }))
  const output = out.texts.find((t) => t.part === 'output')
  expect(output?.text).toMatch(/…$/)
  expect((output?.x ?? 0) + measure(output?.text ?? '', 12, false)).toBeLessThanOrEqual(out.width)
})

test('the format on the streamer is never shortened', () => {
  const out = laidOut(group(0), 190, 262)
  expect(out.card.main).toBe('16 bit / 44.1 kHz')
})

test('a dropped speaker shows "not playing" and is faint', () => {
  const out = laidOut(group(1, { speakers: [{ ...speaker('Office'), playing: false, quality: null }] }))
  expect(texts(out, 'speaker-quality')).toEqual(['not playing'])
  expect(out.icons.find((i) => i.kind === 'speaker' && i.ink === 'faint')).toBeDefined()
})

test('fit shortens with "…" to the room', () => {
  expect(fit('Kitchen', 10, 1000, false, measure)).toBe('Kitchen')
  expect(fit('Upstairs Bathroom', 10, 40, false, measure)).toMatch(/^Ups.*…$/)
})

test('the pulse reaches the speakers in turn, as it passes their junctions, then the outputs', () => {
  const out = laidOut(group(3))
  const speakerPorts = out.ports.filter((p) => p.soft && p.ink === 'lossless' && p.x === 36).sort((a, b) => a.y - b.y)
  // Rows branch from one line: each lights up when the pulse passes its row.
  expect(speakerPorts.length).toBe(3)
  for (let i = 1; i < speakerPorts.length; i++) {
    const [above, below] = [speakerPorts[i - 1], speakerPorts[i]] as [Port, Port]
    expect(below.arrive - above.arrive).toBeCloseTo(below.y - above.y)
  }
  const outputPorts = out.ports.filter((p) => p.x > 80)
  for (const port of outputPorts) {
    for (const row of speakerPorts) expect(port.arrive).toBeGreaterThan(row.arrive)
  }
})

test('without a queue the streamer is only as wide as it needs, so the outputs fit beside it', () => {
  const airplay = player({
    source: { brand: 'airplay', name: 'AirPlay', logoUrl: null, via: null },
    signal: { class: 'unknown', source: null, delivered: null, changes: [], hiResLogoUrl: null },
    outputs: [
      { name: 'Example Speaker', kind: 'speaker' },
      { name: 'Subwoofer', kind: 'subwoofer' },
    ],
  })
  const out = laidOut(airplay, 444, 262, false)
  expect(out.direction).toBe('across')
  expect(out.card.width).toBeLessThan(176)
})
