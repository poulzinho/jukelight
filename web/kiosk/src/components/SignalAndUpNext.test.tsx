// Spec: kiosk-display, "Playing screen" (the signal diagram beside "Up
// next") and "Up next".
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

test('track with all data: a diagram from Qobuz to the streamer', () => {
  show()
  expect(screen.getByRole('img', { name: 'Qobuz to Example Player' })).toBeInTheDocument()
  const main = screen.getByTestId('signal-diagram').querySelector('[data-part="streamer-main"]')
  expect(main).toHaveTextContent('16 bit / 44.1 kHz')
})

test('no signal, no diagram', () => {
  show({ signal: null })
  expect(screen.queryByTestId('signal-diagram')).not.toBeInTheDocument()
})

test('queue beside the diagram: two columns, the diagram first', () => {
  show({ queue: upNext(3) })
  const columns = screen.getByTestId('signal-diagram').parentElement as HTMLElement
  expect(columns.className).toMatch(/grid-cols-\[19em/)
  expect(columns.children[1]).toContainElement(screen.getByRole('list', { name: 'Up next' }))
})

test('without a queue the diagram takes the full width', () => {
  show({ queue: null })
  const columns = screen.getByTestId('signal-diagram').parentElement as HTMLElement
  expect(columns.className).toMatch(/grid-cols-1/)
})

test('up next shows five entries; the fifth is hidden in landscape', () => {
  show({ queue: upNext(6) })
  const list = screen.getByRole('list', { name: 'Up next' })
  const items = within(list).getAllByRole('listitem')
  expect(items).toHaveLength(5)
  expect(items[4]).toHaveClass('landscape:hidden')
  expect(items[3]).not.toHaveClass('landscape:hidden')
  expect(within(list).getByText('Next 1')).toBeInTheDocument()
})

test('long title: one line, ending with "…"', () => {
  show({ queue: [{ title: 'A Remarkably Long Song Title That Keeps Going', artists: ['Example Artist'], artUrl: null }] })
  expect(screen.getByText('A Remarkably Long Song Title That Keeps Going')).toHaveClass('truncate')
})

test('no queue, no up next', () => {
  show({ queue: null })
  expect(screen.queryByRole('list', { name: 'Up next' })).not.toBeInTheDocument()
})

test('empty queue, no up next', () => {
  show({ queue: [] })
  expect(screen.queryByRole('list', { name: 'Up next' })).not.toBeInTheDocument()
})
