// Spec: kiosk-display, "Playing screen", "Progress between updates" and
// "Missing artwork" (task 9.3).
import { act, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

const OBSERVED_AT = 1_000_000

beforeEach(() => {
  vi.useFakeTimers()
  vi.setSystemTime(OBSERVED_AT)
})

afterEach(() => {
  vi.useRealTimers()
})

function show(overrides: Parameters<typeof player>[0] = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

test('track with all data', () => {
  show()
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
  expect(screen.getByText('Example Artist')).toBeInTheDocument()
  expect(screen.getByText('Example Album')).toBeInTheDocument()
  expect(screen.getByRole('img', { name: 'Example Album' })).toHaveAttribute(
    'src',
    '/api/v1/art/0123456789abcdef',
  )
  expect(screen.getByRole('progressbar')).toBeInTheDocument()
  expect(screen.getByText('1:00')).toBeInTheDocument()
  expect(screen.getByText('3:20')).toBeInTheDocument()
})

test('several artists are joined', () => {
  show({ track: { title: 'Duet', artists: ['First', 'Second'], album: null } })
  expect(screen.getByText('First, Second')).toBeInTheDocument()
})

test('no position hides the progress', () => {
  show({ position: null })
  expect(screen.queryByRole('progressbar')).not.toBeInTheDocument()
})

test('progress advances while playing', () => {
  show()
  act(() => {
    vi.advanceTimersByTime(10_000)
  })
  expect(screen.getByText('1:10')).toBeInTheDocument()
})

test('progress stays still while paused', () => {
  show({ state: 'paused' })
  act(() => {
    vi.advanceTimersByTime(10_000)
  })
  expect(screen.getByText('1:00')).toBeInTheDocument()
})

test('progress uses the server clock', () => {
  // This device is 5 s behind the server.
  render(<Kiosk player={player()} connection="live" clockOffsetMs={5_000} />)
  expect(screen.getByText('1:05')).toBeInTheDocument()
})

test('progress stops at the end of the track', () => {
  show({ position: { elapsed: 198, duration: 200, observedAt: OBSERVED_AT } })
  act(() => {
    vi.advanceTimersByTime(10_000)
  })
  expect(screen.getAllByText('3:20')).toHaveLength(2)
})

test('missing artwork shows initials', () => {
  show({ artUrl: null })
  expect(screen.getByRole('img', { name: 'Example Album' })).toHaveTextContent('EA')
})

test('missing artwork without album uses the artists', () => {
  show({ artUrl: null, track: { title: 'Song', artists: ['Solo Artist'], album: null } })
  expect(screen.getByRole('img', { name: 'Solo Artist' })).toHaveTextContent('SA')
})
