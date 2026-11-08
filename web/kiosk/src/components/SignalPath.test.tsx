// Spec: kiosk-display, "Signal path", "Signal colours", "Sound box" and
// "Wave motion".
import { act, render, screen, within } from '@testing-library/react'
import { expect, test, vi } from 'vitest'
import type { Player, Signal, Source } from '../api'
import { Kiosk } from '../Kiosk'
import { player, quality } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const path = () => screen.getByTestId('signal-path')
const parts = () => Array.from(path().children).map((part) => part.getAttribute('data-part'))
const box = () => screen.getByTestId('sound-box')

const viaMusicAssistant = (logoUrl: string | null, available = true): Source['via'] => ({
  brand: 'music-assistant',
  name: 'Music Assistant',
  logoUrl,
  available,
})

const qobuzViaMusicAssistant = (logos: boolean, available = true): Source => ({
  brand: 'qobuz',
  name: 'Qobuz',
  logoUrl: logos ? '/api/v1/logos/qobuz?v=1' : null,
  via: viaMusicAssistant(logos ? '/api/v1/logos/music-assistant?v=1' : null, available),
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

// "Signal path"

test('through Music Assistant: source, wave, route, wave, sound box', () => {
  show({ source: qobuzViaMusicAssistant(true), signal: signal({ delivered: quality('FLAC 24/96', null, true) }) })
  expect(parts()).toEqual(['node', 'wave', 'node', 'wave', 'sound'])
  expect(within(path()).getByRole('img', { name: 'Qobuz' })).toHaveAttribute('src', '/api/v1/logos/qobuz?v=1')
  expect(within(path()).getByRole('img', { name: 'Music Assistant' })).toBeInTheDocument()
})

test('native service: source, one wave, sound box', () => {
  show({
    source: { brand: 'spotify', name: 'Spotify', logoUrl: null, via: null },
    signal: signal({ class: 'lossy', source: quality('320 kbps', null, false) }),
  })
  expect(parts()).toEqual(['node', 'wave', 'sound'])
})

test('no logos: an icon and the names', () => {
  show({ source: qobuzViaMusicAssistant(false), signal: signal() })
  const [source, route] = within(path()).getAllByTestId('signal-node')
  expect(source).toHaveTextContent('Qobuz')
  expect(source?.querySelector('svg')).not.toBeNull()
  expect(route).toHaveTextContent('Music Assistant')
})

test('an icon logo keeps the name next to it', () => {
  show({
    source: { brand: 'navidrome', name: 'Navidrome', logoUrl: '/api/v1/logos/navidrome?v=1', via: null },
    signal: signal(),
  })
  const node = within(path()).getByTestId('signal-node')
  expect(within(node).getByRole('img', { name: 'Navidrome' })).toBeInTheDocument()
  expect(within(node).getByText('Navidrome')).toBeInTheDocument()
})

test('route unavailable: the route greyed, both waves grey and still', () => {
  show({
    source: qobuzViaMusicAssistant(false, false),
    signal: signal({ class: 'unknown', source: null }),
  })
  const route = within(path()).getAllByTestId('signal-node')[1]
  expect(route).toHaveTextContent('unavailable')
  expect(route).toHaveAttribute('data-unavailable', 'true')
  expect(within(path()).queryAllByTestId('pulse')).toHaveLength(0)
  expect(path().style.getPropertyValue('--signal')).toBe('var(--color-signal-unknown)')
})

test('Music Assistant unreachable: only its greyed node, one wave', () => {
  show({
    source: { brand: 'music-assistant', name: 'Music Assistant', logoUrl: null, via: viaMusicAssistant(null, false) },
    signal: signal({ class: 'unknown', source: null }),
  })
  expect(parts()).toEqual(['node', 'wave', 'sound'])
  expect(within(path()).getByTestId('signal-node')).toHaveTextContent('Music Assistant unavailable')
})

test('the labels above the waves', () => {
  show({
    source: qobuzViaMusicAssistant(false),
    signal: signal({
      delivered: quality('FLAC 24/96', null, true),
      changes: [{ kind: 'normalised', gainDb: -6.2 }],
    }),
  })
  const [first, second] = within(path()).getAllByTestId('wave-label')
  expect(first).toHaveTextContent('24/96 FLAC')
  expect(second).toHaveTextContent('24/96 normalised')
})

// "Signal colours"

test('Hi-Res: yellow, and a lighter yellow after the route', () => {
  show({ source: qobuzViaMusicAssistant(false), signal: signal() })
  expect(path().style.getPropertyValue('--signal')).toBe('var(--color-signal-hi-res)')
  const [first, second] = within(path()).getAllByTestId('wave')
  expect(first).toHaveAttribute('data-shade', 'main')
  expect(second).toHaveAttribute('data-shade', 'soft')
})

test('upsampled CD quality: teal', () => {
  show({
    signal: signal({
      class: 'lossless',
      source: quality('FLAC 16/44.1', null, false),
      delivered: quality('FLAC 24/96', null, true),
    }),
  })
  expect(path().style.getPropertyValue('--signal')).toBe('var(--color-signal-lossless)')
})

// "Sound box"

test('untouched Hi-Res with the logo', () => {
  show({ signal: signal({ hiResLogoUrl: '/api/v1/logos/hi-res?v=1' }) })
  expect(within(box()).getByRole('img', { name: 'Hi-Res' })).toHaveAttribute('src', '/api/v1/logos/hi-res?v=1')
  expect(within(box()).getByText('24 bit / 96 kHz')).toBeInTheDocument()
  expect(within(box()).getByText('FLAC')).toBeInTheDocument()
  expect(within(box()).getByText('untouched')).toBeInTheDocument()
})

test('Hi-Res without the logo shows the tag', () => {
  show({ signal: signal() })
  expect(within(box()).getByText('HI-RES')).toBeInTheDocument()
})

test('upsampled CD quality: the delivered format, where it came from, no Hi-Res mark', () => {
  show({
    source: qobuzViaMusicAssistant(false),
    signal: signal({
      class: 'lossless',
      source: quality('FLAC 16/44.1', null, false),
      delivered: quality('FLAC 24/96', null, true),
      changes: [{ kind: 'resampled' }],
    }),
  })
  expect(within(box()).getByText('24 bit / 96 kHz')).toBeInTheDocument()
  expect(within(box()).getByText('resampled from 16 / 44.1')).toBeInTheDocument()
  expect(within(box()).queryByText('HI-RES')).toBeNull()
  expect(within(box()).queryByRole('img')).toBeNull()
})

test('every change, in order', () => {
  show({
    source: qobuzViaMusicAssistant(false),
    signal: signal({
      class: 'lossless',
      source: quality('FLAC 16/44.1', null, false),
      delivered: quality('FLAC 24/44.1', null, false),
      changes: [{ kind: 'bitDepth' }, { kind: 'normalised', gainDb: -6 }, { kind: 'dsp' }],
    }),
  })
  expect(within(box()).getByText('24 bit from 16 · normalised (−6 dB) · DSP')).toBeInTheDocument()
})

test('lossy', () => {
  show({ signal: signal({ class: 'lossy', source: quality('320 kbps', null, false) }) })
  expect(within(box()).getByText('320 kbps')).toBeInTheDocument()
  expect(within(box()).getByText('lossy')).toBeInTheDocument()
  expect(within(box()).getByText('untouched')).toBeInTheDocument()
})

test('unknown', () => {
  show({ signal: signal({ class: 'unknown', source: null }) })
  expect(box()).toHaveTextContent('quality not reported')
})

// "Wave motion"

test('playing: a pulse on each wave, only with motion allowed', () => {
  show({ source: qobuzViaMusicAssistant(false), signal: signal() })
  const pulses = within(path()).getAllByTestId('pulse')
  expect(pulses).toHaveLength(2)
  for (const pulse of pulses) expect(pulse.className).toMatch(/motion-safe:animate-signal-pulse/)
})

test('paused: no pulse', () => {
  show({ state: 'paused', signal: signal() })
  expect(within(path()).queryAllByTestId('pulse')).toHaveLength(0)
})

test('one pulse every 8 seconds, each a new one-off animation', () => {
  vi.useFakeTimers()
  try {
    show({ signal: signal() })
    const first = within(path()).getByTestId('pulse')
    act(() => vi.advanceTimersByTime(7_900))
    expect(within(path()).getByTestId('pulse')).toBe(first)
    act(() => vi.advanceTimersByTime(200))
    expect(within(path()).getByTestId('pulse')).not.toBe(first)
  } finally {
    vi.useRealTimers()
  }
})

test('from one end to the other: the second wave starts as the first pulse arrives', () => {
  show({ source: qobuzViaMusicAssistant(false), signal: signal() })
  const [first, second] = within(path()).getAllByTestId('wave')
  const delays = (wave: HTMLElement | undefined) =>
    Array.from(wave?.querySelectorAll<HTMLElement>('[data-pulse]') ?? []).map((part) => part.style.animationDelay)
  expect(delays(first)).toEqual(['0ms', '0ms'])
  expect(delays(second)).toEqual(['2000ms', '2000ms'])
})
