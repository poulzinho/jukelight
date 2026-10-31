// Spec: kiosk-display, "Input screen" and "Idle screen".
import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

const NOW = 10_000_000_000
const MINUTE = 60_000

beforeEach(() => {
  vi.useFakeTimers()
  vi.setSystemTime(NOW)
})

afterEach(() => {
  vi.useRealTimers()
})

function input(startedAt: number | null, overrides: Partial<Player> = {}): Player {
  return player({
    track: null,
    artUrl: null,
    position: null,
    quality: null,
    queue: null,
    session: { label: 'Optical input', available: true },
    input: { kind: 'optical', name: 'Example Input', startedAt },
    ...overrides,
  })
}

function show(snapshot: Player) {
  return render(<Kiosk player={snapshot} connection="live" clockOffsetMs={0} />)
}

test('optical input', () => {
  show(input(NOW - 23 * MINUTE))
  expect(screen.getByRole('timer')).toBeInTheDocument()
  expect(screen.getByRole('img', { name: 'Disc' })).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Example Input' })).toBeInTheDocument()
  expect(screen.getByText('Optical input')).toBeInTheDocument()
  expect(screen.getByText('for 23 min')).toBeInTheDocument()
  expect(screen.queryByRole('progressbar')).not.toBeInTheDocument()
  expect(screen.queryByText('Up next')).not.toBeInTheDocument()
  expect(screen.queryByText(/nothing playing/)).not.toBeInTheDocument()
})

test('just started', () => {
  show(input(NOW - 20_000))
  expect(screen.getByText('for 0 min')).toBeInTheDocument()
})

test('waiting for a signal', () => {
  show(input(null, { state: 'buffering' }))
  expect(screen.getByText('waiting for signal')).toBeInTheDocument()
  expect(screen.queryByText(/^for \d+ min$/)).not.toBeInTheDocument()
})

test('the time follows the clock offset', () => {
  render(<Kiosk player={input(NOW - 23 * MINUTE)} connection="live" clockOffsetMs={2 * MINUTE} />)
  expect(screen.getByText('for 25 min')).toBeInTheDocument()
})

test('stopped with an input shows the idle screen', () => {
  show(input(NOW - MINUTE, { state: 'stopped' }))
  expect(screen.getByText('Example Player · nothing playing')).toBeInTheDocument()
  expect(screen.queryByRole('img', { name: 'Disc' })).not.toBeInTheDocument()
})

test('the disc turns only when motion is allowed', () => {
  show(input(NOW - MINUTE))
  const disc = screen.getByRole('img', { name: 'Disc' })
  // Tailwind's motion-safe variant: no animation with reduced motion.
  expect(disc.getAttribute('class')).toMatch(/(^|\s)motion-safe:animate-disc(\s|$)/)
  expect(disc.getAttribute('class')).not.toMatch(/(^|\s)animate-disc(\s|$)/)
})
