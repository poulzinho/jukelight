// Spec: kiosk-display, "Source badge" and "Hi-Res badge".
import { render, screen, within } from '@testing-library/react'
import { expect, test } from 'vitest'
import type { Player, Source } from '../api'
import { Kiosk } from '../Kiosk'
import { player, quality } from '../test/fixtures'

function show(overrides: Partial<Player> = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

const viaMusicAssistant = (logoUrl: string | null, available = true): Source['via'] => ({
  brand: 'music-assistant',
  name: 'Music Assistant',
  logoUrl,
  available,
})

test('logo supplied: the source logo and "via" the route logo, in one badge', () => {
  show({
    source: {
      brand: 'qobuz',
      name: 'Qobuz',
      logoUrl: '/api/v1/logos/qobuz?v=1',
      via: viaMusicAssistant('/api/v1/logos/music-assistant?v=1'),
    },
  })
  const badge = screen.getByTestId('source-badge')
  expect(within(badge).getByRole('img', { name: 'Qobuz' })).toHaveAttribute(
    'src',
    '/api/v1/logos/qobuz?v=1',
  )
  expect(within(badge).getByText('via')).toBeInTheDocument()
  expect(within(badge).getByRole('img', { name: 'Music Assistant' })).toBeInTheDocument()
  // A wordmark already names the service.
  expect(within(badge).queryByText('Qobuz')).toBeNull()
})

test('no logos: an icon and the names', () => {
  show({ source: { brand: 'qobuz', name: 'Qobuz', logoUrl: null, via: viaMusicAssistant(null) } })
  const badge = screen.getByTestId('source-badge')
  expect(within(badge).getByText('Qobuz')).toBeInTheDocument()
  expect(within(badge).getByText('via')).toBeInTheDocument()
  expect(within(badge).getByText('Music Assistant')).toBeInTheDocument()
  expect(badge.querySelector('svg')).not.toBeNull()
})

test('an icon logo keeps the name next to it', () => {
  show({
    source: { brand: 'navidrome', name: 'Navidrome', logoUrl: '/api/v1/logos/navidrome?v=1', via: null },
  })
  const badge = screen.getByTestId('source-badge')
  expect(within(badge).getByRole('img', { name: 'Navidrome' })).toBeInTheDocument()
  expect(within(badge).getByText('Navidrome')).toBeInTheDocument()
})

test('route unavailable: a greyed, outlined badge before the source', () => {
  show({
    source: {
      brand: 'qobuz',
      name: 'Qobuz',
      logoUrl: null,
      via: viaMusicAssistant(null, false),
    },
  })
  const unavailable = screen.getByTestId('route-unavailable')
  expect(unavailable).toHaveTextContent('Music Assistant unavailable')
  expect(screen.getByTestId('source-badge')).toHaveTextContent('Qobuz')
  expect(screen.getByTestId('source-badge')).not.toHaveTextContent('via')
})

test('Music Assistant unreachable: only its greyed badge', () => {
  show({
    source: {
      brand: 'music-assistant',
      name: 'Music Assistant',
      logoUrl: '/api/v1/logos/music-assistant?v=1',
      via: viaMusicAssistant('/api/v1/logos/music-assistant?v=1', false),
    },
  })
  const unavailable = screen.getByTestId('route-unavailable')
  expect(within(unavailable).getByRole('img', { name: 'Music Assistant' })).toBeInTheDocument()
  expect(unavailable).toHaveTextContent('unavailable')
  expect(screen.queryByTestId('source-badge')).toBeNull()
})

test('Hi-Res track with the logo supplied shows the logo instead of the tag', () => {
  show({ quality: { ...quality('FLAC 24/96', null, true), hiResLogoUrl: '/api/v1/logos/hi-res?v=1' } })
  expect(screen.getByRole('img', { name: 'Hi-Res' })).toHaveAttribute('src', '/api/v1/logos/hi-res?v=1')
  expect(screen.queryByText('HI-RES')).toBeNull()
})
