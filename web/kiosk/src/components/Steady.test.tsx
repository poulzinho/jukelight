// Spec: kiosk-display, "Update only what changed".
import { act, render, screen } from '@testing-library/react'
import { beforeEach, expect, test, vi } from 'vitest'
import { Kiosk } from '../Kiosk'
import type { Player } from '../api'
import { loadImage } from '../loadImage'
import { player } from '../test/fixtures'

vi.mock('../loadImage', () => ({ loadImage: vi.fn() }))

/** Covers load when the test says so. */
let loaded: () => void = () => {}
beforeEach(() => {
  vi.mocked(loadImage).mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        loaded = resolve
      }),
  )
})

function screenFor(first: Player) {
  const view = render(<Kiosk player={first} connection="live" clockOffsetMs={0} />)
  return (next: Player) => view.rerender(<Kiosk player={next} connection="live" clockOffsetMs={0} />)
}

const cover = (name: string) => screen.getByRole('img', { name })

test('next track on the same album: the cover stays, the title fades in', () => {
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
  const newTitle = screen.getByRole('heading', { name: 'Second Song' })
  expect(newTitle).not.toBe(title)
  expect(newTitle.closest('.motion-safe\\:animate-fade-in')).not.toBeNull()
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

test('quality changes: only the badges change, nothing is rebuilt', () => {
  const update = screenFor(player())
  const before = cover('Example Album')
  const title = screen.getByRole('heading', { name: 'Example Song' })

  update(player({ quality: { label: 'FLAC 24/96', detail: 'Hi-Res', hiRes: true } }))
  expect(screen.getByText('FLAC 24/96 · Hi-Res')).toBeInTheDocument()
  expect(cover('Example Album')).toBe(before)
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBe(title)
})
