// Spec: kiosk-display, "Signal diagram", "Streamer card", "Line colours" and
// "Pulse along the diagram".
import { render, screen } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player, Signal, Source } from '../api'
import { Kiosk } from '../Kiosk'
import { player, quality } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const diagram = () => screen.getByTestId('signal-diagram')
const parts = (part: string) => [...diagram().querySelectorAll(`[data-part="${part}"]`)]
const texts = (part: string) => parts(part).map((element) => element.textContent)

const qobuzViaMusicAssistant = (available = true): Source => ({
  brand: 'qobuz',
  name: 'Qobuz',
  logoUrl: '/api/v1/logos/qobuz?v=1',
  via: { brand: 'music-assistant', name: 'Music Assistant', logoUrl: '/api/v1/logos/music-assistant?v=1', available },
})

function signal(overrides: Partial<Signal> = {}): Signal {
  return {
    class: 'hi-res',
    source: quality('FLAC 24/96', null, true),
    delivered: null,
    changes: [],
    hiResLogoUrl: null,
    ...overrides,
  }
}

const speaker = (name: string, overrides: Partial<Player['speakers'][number]> = {}): Player['speakers'][number] => ({
  name,
  kind: 'speaker',
  playing: true,
  quality: { ...quality('PCM 16/44.1', null, false), class: 'lossless' },
  ...overrides,
})

const outputs: Player['outputs'] = [
  { name: 'Example Speaker', kind: 'speaker' },
  { name: 'Subwoofer', kind: 'subwoofer' },
]

const queue = [{ title: 'Next Song', artists: ['Example Artist'], artUrl: null }]

// "Signal diagram"

test('a group with a queue: logos, rows, the streamer and its outputs, top to bottom', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal({ delivered: quality('FLAC 24/96', null, true) }),
    speakers: [speaker('Kitchen'), speaker('Office', { kind: 'computer' })],
    outputs,
    queue,
  })
  expect(diagram()).toHaveAttribute('data-direction', 'down')
  expect(parts('source')[0]).toHaveAttribute('href', '/api/v1/logos/qobuz?v=1')
  expect(parts('route')[0]).toHaveAttribute('href', '/api/v1/logos/music-assistant?v=1')
  expect(texts('streams')).toEqual(['24/96'])
  expect(texts('streams-codec')).toEqual(['FLAC'])
  expect(texts('speaker-name')).toEqual(['Kitchen', 'Office'])
  expect(texts('speaker-quality')).toEqual(['PCM', '16/44.1', 'PCM', '16/44.1'])
  expect(parts('icon-computer')).toHaveLength(1)
  expect(texts('output')).toEqual(['Example Speaker', 'Subwoofer'])
  expect(parts('icon-subwoofer')).toHaveLength(1)
})

test('native service without a queue: across the width', () => {
  show({
    source: { brand: 'spotify', name: 'Spotify', logoUrl: null, via: null },
    signal: signal({ class: 'lossy', source: quality('320 kbps', null, false) }),
    outputs: [{ name: 'Subwoofer', kind: 'subwoofer' }],
    queue: null,
  })
  expect(diagram()).toHaveAttribute('data-direction', 'across')
  expect(texts('streams')).toEqual(['320 kbps'])
  expect(texts('streams-codec')).toEqual(['lossy'])
})

test('AirPlay from a phone: no label on the line, the streamer says "AirPlay"', () => {
  show({
    source: { brand: 'airplay', name: 'AirPlay', logoUrl: null, via: null },
    signal: signal({ class: 'unknown', source: null }),
    queue: null,
  })
  expect(texts('streams')).toEqual([])
  expect(texts('source')).toEqual(['AirPlay'])
  expect(texts('streamer-main')).toEqual(['AirPlay'])
})

test('Music Assistant unavailable: its logo greyed with "unavailable", the streamer shows only the room', () => {
  show({
    source: {
      brand: 'music-assistant',
      name: 'Music Assistant',
      logoUrl: '/api/v1/logos/music-assistant?v=1',
      via: { brand: 'music-assistant', name: 'Music Assistant', logoUrl: '/api/v1/logos/music-assistant?v=1', available: false },
    },
    signal: signal({ class: 'unknown', source: null }),
    queue: null,
  })
  expect(parts('source')[0]).toHaveAttribute('opacity', '0.4')
  expect(texts('source')).toContain('unavailable')
  expect(texts('streamer-main')).toEqual(['Example Player'])
})

test('a speaker dropped out: dimmed, "not playing"', () => {
  show({ source: qobuzViaMusicAssistant(), signal: signal(), speakers: [speaker('Office', { playing: false, quality: null })], queue })
  expect(texts('speaker-quality')).toEqual(['not playing'])
})

// "Streamer card"

test('Hi-Res with both logos: the Hi-Res logo, the format, the codec, the room and the mark', () => {
  show({
    signal: signal({ hiResLogoUrl: '/api/v1/logos/hi-res?v=1' }),
    device: { brand: 'bluesound', name: 'Example Player', logoUrl: null, markUrl: '/api/v1/logos/bluesound-mark?v=1' },
    queue,
  })
  const card = parts('streamer')[0] as Element
  expect(card.querySelector('image[href="/api/v1/logos/hi-res?v=1"]')).not.toBeNull()
  expect(card.querySelector('image[href="/api/v1/logos/bluesound-mark?v=1"]')).not.toBeNull()
  expect(texts('streamer-main')).toEqual(['24 bit / 96 kHz'])
  expect(texts('streamer-codec')).toEqual(['FLAC · untouched'])
  expect(texts('streamer-room')).toEqual(['Example Player'])
})

test('Hi-Res without the logo shows the tag', () => {
  show({ signal: signal(), queue })
  expect((parts('streamer')[0] as Element).textContent).toContain('HI-RES')
})

test('a group member receiving PCM: the PCM icon with "PCM"', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal({ class: 'lossless', source: quality('FLAC 16/44.1', null, false), delivered: quality('PCM 16/44.1', null, false) }),
    queue,
  })
  expect(texts('streamer-main')).toEqual(['16 bit / 44.1 kHz'])
  expect(texts('streamer-codec')).toEqual(['PCM · untouched'])
  expect((parts('streamer')[0] as Element).querySelector('[data-part="pcm"]')).not.toBeNull()
})

test('a lossy source decoded to PCM', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal({ class: 'lossy', source: quality('320 kbps', null, false), delivered: quality('PCM 16/44.1', null, false) }),
    queue,
  })
  expect(texts('streamer-codec')).toEqual(['PCM · decoded from 320 kbps'])
})

// "Line colours"

test('one lossy speaker in a lossless group: its line violet, the others teal', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal({ class: 'lossless', source: quality('FLAC 16/44.1', null, false) }),
    speakers: [speaker('Kitchen'), speaker('Garden', { quality: { ...quality('256 kbps', null, false), class: 'lossy' } })],
    queue,
  })
  const strokes = [...diagram().querySelectorAll('path')].map((path) => path.getAttribute('stroke'))
  expect(strokes).toContain('var(--color-signal-lossy-soft)')
  expect(strokes).toContain('var(--color-signal-lossless-soft)')
})

test('upsampled CD quality: teal, no Hi-Res logo', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal({ class: 'lossless', source: quality('FLAC 16/44.1', null, false), delivered: quality('FLAC 24/96', null, true) }),
    queue,
  })
  const card = parts('streamer')[0] as Element
  expect(card.querySelector('rect')).toHaveAttribute('stroke', 'var(--color-signal-lossless-soft)')
  expect(card.textContent).not.toContain('HI-RES')
})

// "Pulse along the diagram"

test('a track starts in a group: one pulse along every line, from the source on', () => {
  show({ source: qobuzViaMusicAssistant(), signal: signal(), speakers: [speaker('Kitchen'), speaker('Office')], outputs, queue })
  const sparks = parts('spark') as HTMLElement[]
  expect(sparks.length).toBeGreaterThan(5)
  const delays = sparks.map((spark) => parseInt(spark.style.animationDelay, 10))
  expect(delays[0]).toBe(0)
  // Steps, about 15 a second.
  expect(sparks.every((spark) => spark.style.animationTimingFunction.startsWith('steps('))).toBe(true)
})

test('paused: no pulse', () => {
  show({ state: 'paused', signal: signal(), queue })
  expect(screen.queryByTestId('pulse')).toBeNull()
})

test('nothing changes: the same pulse stays, no new one', () => {
  const { rerender } = show({ signal: signal(), queue })
  const first = screen.getByTestId('pulse')
  rerender(<Kiosk player={player({ signal: signal(), queue, position: { elapsed: 120, duration: 200, observedAt: 1_060_000 } })} connection="live" clockOffsetMs={0} />)
  expect(screen.getByTestId('pulse')).toBe(first)
})

test('the quality changes: a new pulse', () => {
  const { rerender } = show({ signal: signal(), queue })
  const first = screen.getByTestId('pulse')
  rerender(<Kiosk player={player({ signal: signal({ class: 'lossless', source: quality('FLAC 16/44.1', null, false) }), queue })} connection="live" clockOffsetMs={0} />)
  expect(screen.getByTestId('pulse')).not.toBe(first)
})

// "Streamer card": the lid holds the mark and the room, set like a speaker.

test('the room in the lid, set like a speaker\'s name, after the mark', () => {
  show({
    source: qobuzViaMusicAssistant(),
    signal: signal(),
    speakers: [speaker('Kitchen')],
    device: { brand: 'bluesound', name: 'Example Player', logoUrl: null, markUrl: '/api/v1/logos/bluesound-mark?v=1' },
    queue,
  })
  const room = parts('streamer-room')[0] as SVGTextElement
  const name = parts('speaker-name')[0] as SVGTextElement
  const main = parts('streamer-main')[0] as SVGTextElement
  for (const attribute of ['font-size', 'font-weight', 'fill']) {
    expect(room.getAttribute(attribute), attribute).toBe(name.getAttribute(attribute))
  }
  expect(Number(room.getAttribute('y'))).toBeLessThan(Number(main.getAttribute('y')))
  const mark = (parts('streamer')[0] as Element).querySelector('image[href="/api/v1/logos/bluesound-mark?v=1"]')
  expect(Number(mark?.getAttribute('x'))).toBeLessThan(Number(room.getAttribute('x')))
})

test('no mark: the room alone in the lid', () => {
  show({ signal: signal(), device: { brand: null, name: 'Example Player', logoUrl: null, markUrl: null }, queue })
  const card = parts('streamer')[0] as Element
  expect(card.querySelector('image')).toBeNull()
  expect(texts('streamer-room')).toEqual(['Example Player'])
})

// "Line colours": the lines stay in the background.

test('lines, ports and dots are thin and partly transparent; the card\'s border is as heavy as the icons', () => {
  show({ source: qobuzViaMusicAssistant(), signal: signal(), speakers: [speaker('Kitchen')], outputs, queue })
  // The diagram's own lines: the paths directly in its drawing.
  const lines = [...(diagram().querySelector('svg') as SVGSVGElement).querySelectorAll(':scope > path')]
  expect(lines.length).toBeGreaterThan(0)
  for (const line of lines) {
    expect(Number(line.getAttribute('stroke-width'))).toBe(1)
    expect(Number(line.getAttribute('stroke-opacity'))).toBe(0.6)
  }
  const border = (parts('streamer')[0] as Element).querySelector('rect')
  expect(Number(border?.getAttribute('stroke-width'))).toBe(1.05)
  const icon = parts('icon-speaker')[0] as Element
  expect(Number(icon.getAttribute('stroke-width'))).toBe(1.4)
})

// "Pulse along the diagram": the wave lights up as a pulse starts.

test('a pulse lights up the wave and its format at once', () => {
  show({ source: { ...qobuzViaMusicAssistant(), logoUrl: '/api/v1/logos/qobuz?v=1' }, signal: signal(), speakers: [speaker('Kitchen')], queue })
  const flash = parts('wave-flash')
  expect(flash).toHaveLength(1)
  expect(flash[0]).toHaveClass('signal-flash')
  expect((flash[0] as HTMLElement).style.animationDelay).toBe('0ms')
})

test('the flash text lies exactly over the format: set the same way', () => {
  show({ source: { ...qobuzViaMusicAssistant(), logoUrl: '/api/v1/logos/qobuz?v=1' }, signal: signal(), speakers: [speaker('Kitchen')], queue })
  const flash = parts('wave-flash')[0] as Element
  // The bright wave lies exactly over the wave: the same path and width.
  const wave = [...(diagram().querySelector('svg') as SVGSVGElement).querySelectorAll(':scope > path')].find((p) => p.getAttribute('d')?.includes(' q'))
  const brightWave = flash.querySelector('path')
  expect(brightWave?.getAttribute('d')).toBe(wave?.getAttribute('d'))
  expect(brightWave?.getAttribute('stroke-width')).toBe(wave?.getAttribute('stroke-width'))
  const copies = [...flash.querySelectorAll('text')]
  const originals = [...parts('streams'), ...parts('streams-codec')]
  expect(copies).toHaveLength(originals.length)
  for (const original of originals) {
    const copy = copies.find((text) => text.textContent === original.textContent)
    for (const attribute of ['x', 'y', 'font-size', 'font-weight', 'text-anchor', 'class']) {
      expect(copy?.getAttribute(attribute), `${original.textContent} ${attribute}`).toBe(original.getAttribute(attribute))
    }
  }
})

test('no quality reported: nothing to light up', () => {
  show({ source: { brand: 'airplay', name: 'AirPlay', logoUrl: null, via: null }, signal: signal({ class: 'unknown', source: null }), queue: null })
  expect(parts('wave-flash')).toHaveLength(0)
})
