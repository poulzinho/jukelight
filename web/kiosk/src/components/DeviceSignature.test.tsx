// Spec: kiosk-display, "Device signature".
import { render, screen, within } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const withLogo = { brand: 'bluesound', name: 'Example Player', logoUrl: '/api/v1/logos/bluesound?v=1' }
const screens: [string, Partial<Player>][] = [
  ['playing', {}],
  ['input', { track: null, input: { kind: 'optical', name: 'Example Input', startedAt: null } }],
  ['idle', { state: 'stopped', track: null }],
]

for (const [name, overrides] of screens) {
  test(`${name} screen: the player's logo and name`, () => {
    show({ ...overrides, device: withLogo })
    const signature = screen.getByTestId('device-signature')
    // The logo is decorative: the name next to it says which player it is.
    expect(signature.querySelector('img')).toHaveAttribute('src', '/api/v1/logos/bluesound?v=1')
    expect(within(signature).getByText('Example Player')).toBeInTheDocument()
  })
}

test('no logo: a streamer icon and the maker\'s name', () => {
  show({ device: { brand: 'bluesound', name: 'Example Player', logoUrl: null } })
  const signature = screen.getByTestId('device-signature')
  expect(signature.querySelector('img')).toBeNull()
  expect(signature.querySelector('svg')).not.toBeNull()
  expect(within(signature).getByText('Bluesound')).toBeInTheDocument()
  expect(within(signature).getByText('Example Player')).toBeInTheDocument()
})

test('unknown maker: only the name', () => {
  show({ device: { brand: null, name: 'Example Player', logoUrl: null } })
  const signature = screen.getByTestId('device-signature')
  expect(signature).toHaveTextContent(/^Example Player$/)
  expect(signature.querySelector('svg, img')).toBeNull()
})
