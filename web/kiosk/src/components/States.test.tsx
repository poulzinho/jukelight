// Spec: kiosk-display, "Paused screen", "Idle screen" and "Offline and
// reconnecting" (task 9.5).
import { render, screen, within } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import type { Connection } from '../usePlayer'
import { player } from '../test/fixtures'

function show(overrides: Partial<Player> = {}, connection: Connection = 'live') {
  return render(<Kiosk player={player(overrides)} connection={connection} clockOffsetMs={0} />)
}

test('pause shows the pause symbol over dimmed content', () => {
  show({ state: 'paused' })
  expect(screen.getByRole('status', { name: 'Paused' })).toBeInTheDocument()
  expect(screen.getByTestId('dim')).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
})

test('playing has no pause symbol', () => {
  show()
  expect(screen.queryByRole('status', { name: 'Paused' })).not.toBeInTheDocument()
  expect(screen.queryByTestId('dim')).not.toBeInTheDocument()
})

test('stopped shows the idle screen', () => {
  show({ state: 'stopped' })
  expect(screen.getByText('nothing playing')).toBeInTheDocument()
  expect(within(screen.getByTestId('device-signature')).getByText('Example Player')).toBeInTheDocument()
  expect(screen.getByRole('timer')).toBeInTheDocument()
  expect(screen.queryByRole('heading', { name: 'Example Song' })).not.toBeInTheDocument()
})

test('no track shows the idle screen', () => {
  show({ track: null })
  expect(screen.getByText('nothing playing')).toBeInTheDocument()
  expect(within(screen.getByTestId('device-signature')).getByText('Example Player')).toBeInTheDocument()
})

test('no snapshot yet shows the idle screen', () => {
  render(<Kiosk player={null} connection="connecting" clockOffsetMs={0} />)
  expect(screen.getByText('nothing playing')).toBeInTheDocument()
})

test('buffering keeps the playing screen', () => {
  show({ state: 'buffering' })
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
})

test('lost connection keeps the last screen and shows the indicator', () => {
  show({}, 'lost')
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
  expect(screen.getByRole('alert')).toHaveTextContent('Reconnecting')
})

test('offline player keeps the last screen and shows the indicator', () => {
  show({ online: false })
  expect(screen.getByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
  expect(screen.getByRole('alert')).toHaveTextContent('Player offline')
})

test('live and online has no indicator', () => {
  show()
  expect(screen.queryByRole('alert')).not.toBeInTheDocument()
})
