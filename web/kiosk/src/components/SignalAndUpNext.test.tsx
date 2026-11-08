// Spec: kiosk-display, "Playing screen" (the signal path) and "Up next".
import { render, screen, within } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const upNext = (count: number) =>
  Array.from({ length: count }, (_, i) => ({
    title: `Next ${i + 1}`,
    artists: ['Example Artist'],
    artUrl: null,
  }))

test('track with all data: a signal path from Qobuz to the sound box', () => {
  show()
  const path = screen.getByTestId('signal-path')
  expect(within(path).getByTestId('signal-node')).toHaveTextContent('Qobuz')
  const box = screen.getByTestId('sound-box')
  expect(within(box).getByText('16 bit / 44.1 kHz')).toBeInTheDocument()
  expect(within(box).getByText('FLAC')).toBeInTheDocument()
})

test('no signal, no signal path', () => {
  show({ signal: null })
  expect(screen.queryByTestId('signal-path')).not.toBeInTheDocument()
})

test('up next shows four entries; the fourth is hidden in portrait', () => {
  show({ queue: upNext(5) })
  const list = screen.getByRole('list', { name: 'Up next' })
  const items = within(list).getAllByRole('listitem')
  expect(items).toHaveLength(4)
  expect(items[3]).toHaveClass('portrait:hidden')
  expect(items[2]).not.toHaveClass('portrait:hidden')
  expect(within(list).getByText('Next 1')).toBeInTheDocument()
})

test('no queue, no up next', () => {
  show({ queue: null })
  expect(screen.queryByRole('list', { name: 'Up next' })).not.toBeInTheDocument()
})

test('empty queue, no up next', () => {
  show({ queue: [] })
  expect(screen.queryByRole('list', { name: 'Up next' })).not.toBeInTheDocument()
})
