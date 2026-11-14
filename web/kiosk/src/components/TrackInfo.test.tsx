// Spec: kiosk-display, "Playing screen": the stepped title, the artists on
// up to two lines, and the icons by the artists and the album.
import { render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, test } from 'vitest'
import type { Player } from '../api'
import { TrackInfo } from './TrackInfo'

/**
 * A stand-in for the browser's layout: text runs at half its font size per
 * character across a column 74 units wide (the portrait column, in vmin),
 * so the title overflows when it needs more lines than it may show.
 */
const COLUMN = 74
function lines(element: HTMLElement): { needed: number; shown: number; height: number } {
  const size = Number.parseFloat(element.style.fontSize) || 7.2
  const clamp = Number(element.style.webkitLineClamp || element.style.getPropertyValue('-webkit-line-clamp')) || 1
  const needed = Math.max(1, Math.ceil(((element.textContent ?? '').length * size * 0.5) / COLUMN))
  return { needed, shown: Math.min(needed, clamp), height: size * 1.02 }
}
const descriptors: [string, PropertyDescriptor | undefined][] = []
beforeEach(() => {
  for (const name of ['scrollHeight', 'clientHeight']) {
    descriptors.push([name, Object.getOwnPropertyDescriptor(HTMLElement.prototype, name)])
  }
  Object.defineProperty(HTMLElement.prototype, 'scrollHeight', {
    configurable: true,
    get(this: HTMLElement) {
      if (this.tagName !== 'H1') return 0
      const { needed, height } = lines(this)
      return needed * height
    },
  })
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    get(this: HTMLElement) {
      if (this.tagName !== 'H1') return 0
      const { shown, height } = lines(this)
      return shown * height
    },
  })
})
afterEach(() => {
  for (const [name, descriptor] of descriptors.splice(0)) {
    if (descriptor) Object.defineProperty(HTMLElement.prototype, name, descriptor)
  }
})

const track = (overrides: Partial<NonNullable<Player['track']>> = {}): NonNullable<Player['track']> => ({
  title: 'Example Song',
  artists: ['Example Artist'],
  album: 'Example Album',
  ...overrides,
})

function title() {
  const heading = screen.getByRole('heading', { level: 1 })
  return { size: Number.parseFloat(heading.style.fontSize), lines: Number(heading.style.webkitLineClamp) }
}

test('short title: the largest size, one line', () => {
  render(<TrackInfo track={track()} />)
  expect(title()).toEqual({ size: 7.2, lines: 1 })
})

test('medium title: two lines at a smaller size', () => {
  render(<TrackInfo track={track({ title: 'Example Song Title (Remastered 2024)' })} />)
  const { size, lines } = title()
  expect(lines).toBe(2)
  expect(size).toBeLessThan(6.2)
})

test('long classical title: in full, smaller, without an ellipsis', () => {
  render(<TrackInfo track={track({ title: 'J.S. Bach: Violin Concerto No. 1 in A Minor, BWV 1041: I. Allegro moderato' })} />)
  const heading = screen.getByRole('heading', { level: 1 })
  expect(title().size).toBeLessThan(5.4)
  // It fits: what it needs is all it shows.
  expect(heading.scrollHeight).toBe(heading.clientHeight)
})

test('title too long for any size: three lines at the smallest size', () => {
  render(<TrackInfo track={track({ title: 'An Example Title '.repeat(20) })} />)
  expect(title()).toEqual({ size: 3.6, lines: 3 })
})

test('two artists on two lines: in full, up to two lines', () => {
  render(<TrackInfo track={track({ artists: ['Example Violinist', 'Example Chamber Orchestra'] })} />)
  const artists = screen.getByText('Example Violinist, Example Chamber Orchestra')
  expect(artists).toHaveClass('line-clamp-2')
})

test('artists and album with icons: one person, a disc', () => {
  render(<TrackInfo track={track()} />)
  expect(screen.getByTestId('artist-icon')).toHaveAttribute('data-kind', 'person')
  expect(screen.getByTestId('album-icon')).toBeInTheDocument()
})

test.each([
  ['several artists', ['Example Singer', 'Guest Artist']],
  ["a group's name", ['Example Chamber Orchestra']],
  ['a group word in another language', ['Orquesta de Ejemplo']],
])('%s: a group icon', (_, artists) => {
  render(<TrackInfo track={track({ artists })} />)
  expect(screen.getByTestId('artist-icon')).toHaveAttribute('data-kind', 'group')
})

test('a name without a group word: one person', () => {
  render(<TrackInfo track={track({ artists: ['The Example Collective'] })} />)
  expect(screen.getByTestId('artist-icon')).toHaveAttribute('data-kind', 'person')
})

test('a group word only as part of a longer word: one person', () => {
  render(<TrackInfo track={track({ artists: ['Example Bandit'] })} />)
  expect(screen.getByTestId('artist-icon')).toHaveAttribute('data-kind', 'person')
})
