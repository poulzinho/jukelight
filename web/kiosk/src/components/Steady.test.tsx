// Spec: kiosk-display, "Update only what changed".
import { act, fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, expect, test, vi } from 'vitest'
import { Kiosk } from '../Kiosk'
import type { Player } from '../api'
import { loadImage } from '../loadImage'
import { player, quality } from '../test/fixtures'

vi.mock('../loadImage', () => ({ loadImage: vi.fn() }))

/** Covers load, or fail, when the test says so. */
let loaded: () => void = () => {}
let failed: () => void = () => {}
beforeEach(() => {
  vi.mocked(loadImage).mockReset()
  vi.mocked(loadImage).mockImplementation(
    () =>
      new Promise<void>((resolve, reject) => {
        loaded = resolve
        failed = () => reject(new Error('not an image'))
      }),
  )
})

function screenFor(first: Player) {
  const view = render(<Kiosk player={first} connection="live" clockOffsetMs={0} />)
  return (next: Player) => view.rerender(<Kiosk player={next} connection="live" clockOffsetMs={0} />)
}

const cover = (name: string) => screen.getByRole('img', { name })

test('next track on the same album: the cover stays, the title fades in', async () => {
  const update = screenFor(player())
  const before = cover('Example Album')
  const title = screen.getByRole('heading', { name: 'Example Song' })

  update(
    player({
      track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' },
      // BluOS gives each song its own image URL.
      artUrl: '/api/v1/art/fedcba9876543210?size=large',
      backdropUrl: '/api/v1/art/fedcba9876543210?size=backdrop',
    }),
  )
  expect(cover('Example Album')).toBe(before)
  expect(loadImage).not.toHaveBeenCalled()
  const newTitle = await screen.findByRole('heading', { name: 'Second Song' })
  expect(newTitle).not.toBe(title)
  expect(newTitle.closest('.motion-safe\\:animate-rise-in')).not.toBeNull()
  expect(cover('Example Album')).toBe(before)
})

test('old text fades out first, then the new text replaces it', async () => {
  const update = screenFor(player())
  update(player({ track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' } }))
  const old = screen.getByRole('heading', { name: 'Example Song' })
  expect(old.closest('.motion-safe\\:animate-fade-out')).not.toBeNull()
  expect(screen.queryByRole('heading', { name: 'Second Song' })).toBeNull()

  expect(await screen.findByRole('heading', { name: 'Second Song' })).toBeInTheDocument()
  expect(screen.queryByRole('heading', { name: 'Example Song' })).toBeNull()
})

test('snapshots keep arriving during the change: the new text still replaces the old', () => {
  vi.useFakeTimers()
  try {
    const update = screenFor(player())
    const next = { track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' } }
    update(player(next))
    // A new snapshot every 200 ms, with the same track, for two seconds.
    for (let i = 0; i < 10; i++) {
      act(() => vi.advanceTimersByTime(200))
      update(player(next))
    }
    expect(screen.getByRole('heading', { name: 'Second Song' })).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'Example Song' })).toBeNull()
  } finally {
    vi.useRealTimers()
  }
})

test('reduced motion: the new text replaces the old at once, with the newest content', () => {
  vi.useFakeTimers()
  const matchMedia = window.matchMedia
  window.matchMedia = ((query: string) => ({ matches: query.includes('reduce'), media: query })) as typeof window.matchMedia
  try {
    const update = screenFor(player())
    update(player({ track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' } }))
    update(player({ track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' } }))
    act(() => vi.advanceTimersByTime(0))
    expect(screen.getByRole('heading', { name: 'Second Song' })).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'Example Song' })).toBeNull()
  } finally {
    window.matchMedia = matchMedia
    vi.useRealTimers()
  }
})

test('track with a new cover: the old one stays until the new one has loaded', async () => {
  const update = screenFor(player())
  update(
    player({
      track: { title: 'Other Song', artists: ['Other Artist'], album: 'Other Album' },
      artUrl: '/api/v1/art/1111111111111111?size=large',
      backdropUrl: '/api/v1/art/1111111111111111?size=backdrop',
    }),
  )
  expect(cover('Example Album')).toBeInTheDocument()
  expect(screen.queryByRole('img', { name: 'Other Album' })).toBeNull()

  await act(async () => loaded())
  expect(cover('Other Album')).toHaveAttribute('src', '/api/v1/art/1111111111111111?size=large')
})

test('quality changes: only the diagram changes, in place', () => {
  const update = screenFor(player())
  const before = cover('Example Album')
  const title = screen.getByRole('heading', { name: 'Example Song' })
  const diagram = screen.getByTestId('signal-diagram')

  const hiRes = quality('FLAC 24/96', 'Hi-Res', true)
  update(player({ quality: hiRes, signal: { class: 'hi-res', source: hiRes, delivered: null, changes: [], hiResLogoUrl: null } }))
  expect(diagram.querySelector('[data-part="streamer-main"]')).toHaveTextContent('24 bit / 96 kHz')
  expect(screen.getByText('HI-RES')).toBeInTheDocument()
  expect(screen.getByTestId('signal-diagram')).toBe(diagram)
  expect(cover('Example Album')).toBe(before)
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBe(title)
})

const otherAlbum = player({
  track: { title: 'Other Song', artists: ['Other Artist'], album: 'Other Album' },
  artUrl: '/api/v1/art/1111111111111111?size=large',
  backdropUrl: '/api/v1/art/1111111111111111?size=backdrop',
})

test('a cover that fails to load shows the initials, not a broken image', async () => {
  const update = screenFor(player())
  update(otherAlbum)
  await act(async () => failed())
  expect(cover('Other Album')).toHaveTextContent('OA')
  expect(cover('Other Album').tagName).not.toBe('IMG')
})

test('a failed backdrop tries again when a new URL arrives for the same album', async () => {
  const update = screenFor(player())
  const image = screen.getByTestId('backdrop').querySelector('img')
  if (!image) throw new Error('no backdrop image')
  fireEvent.error(image)
  expect(screen.getByTestId('backdrop').querySelector('img')).toBeNull()

  update(
    player({
      track: { title: 'Second Song', artists: ['Example Artist'], album: 'Example Album' },
      backdropUrl: '/api/v1/art/fedcba9876543210?size=backdrop',
    }),
  )
  expect(loadImage).toHaveBeenCalledWith('/api/v1/art/fedcba9876543210?size=backdrop')
  await act(async () => loaded())
  expect(screen.getByTestId('backdrop').querySelector('img')).toHaveAttribute(
    'src',
    '/api/v1/art/fedcba9876543210?size=backdrop',
  )
})
