// Spec: kiosk-display, "Playing screen" (badges), "Hi-Res badge" and
// "Up next" (task 9.4).
import { render, screen, within } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import { player, quality } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const upNext = (count: number) =>
  Array.from({ length: count }, (_, i) => ({
    title: `Next ${i + 1}`,
    artists: ['Example Artist'],
    artUrl: null,
  }))

test('session and quality badges', () => {
  show()
  expect(screen.getByText('Qobuz')).toBeInTheDocument()
  expect(screen.getByText('FLAC 16/44.1 · CD quality')).toBeInTheDocument()
})

test('unavailable session', () => {
  show({ session: { label: 'Example Provider', available: false } })
  expect(screen.getByText('Example Provider unavailable')).toBeInTheDocument()
})

test('quality without detail', () => {
  show({ quality: quality('160 kbps', null, false) })
  expect(screen.getByText('160 kbps')).toBeInTheDocument()
})

test('Hi-Res track shows the HI-RES badge', () => {
  show({ quality: quality('FLAC 24/96', null, true) })
  expect(screen.getByText('HI-RES')).toBeInTheDocument()
})

test('CD-quality track has no HI-RES badge', () => {
  show()
  expect(screen.queryByText('HI-RES')).not.toBeInTheDocument()
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
