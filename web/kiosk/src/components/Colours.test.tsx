// Spec: kiosk-display, "Album colours". The page and the progress bar take
// their colours from --bg and --accent (see index.css); the browser test
// checks the computed colours.
import { render } from '@testing-library/react'
import { afterEach, expect, test } from 'vitest'
import type { Player } from '../api'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

const root = document.documentElement

afterEach(() => {
  root.style.removeProperty('--bg')
  root.style.removeProperty('--accent')
})

function show(palette: Player['palette']) {
  return render(<Kiosk player={player({ palette })} connection="live" clockOffsetMs={0} />)
}

test('colours known', () => {
  show({ background: [73, 51, 38], accent: [209, 147, 108] })
  expect(root.style.getPropertyValue('--bg')).toBe('73 51 38')
  expect(root.style.getPropertyValue('--accent')).toBe('209 147 108')
})

test('colours unknown', () => {
  show(null)
  // Unset, so the defaults in index.css apply (background rgb(18, 18, 22)).
  expect(root.style.getPropertyValue('--bg')).toBe('')
  expect(root.style.getPropertyValue('--accent')).toBe('')
})

test('colours change with the track', () => {
  const { rerender } = show({ background: [73, 51, 38], accent: [209, 147, 108] })
  rerender(
    <Kiosk
      player={player({ palette: { background: [10, 20, 30], accent: [200, 210, 220] } })}
      connection="live"
      clockOffsetMs={0}
    />,
  )
  expect(root.style.getPropertyValue('--bg')).toBe('10 20 30')
  expect(root.style.getPropertyValue('--accent')).toBe('200 210 220')

  rerender(<Kiosk player={player({ palette: null })} connection="live" clockOffsetMs={0} />)
  expect(root.style.getPropertyValue('--bg')).toBe('')
})

test('the idle screen uses the default colours', () => {
  const { rerender } = show({ background: [73, 51, 38], accent: [209, 147, 108] })
  rerender(<Kiosk player={null} connection="live" clockOffsetMs={0} />)
  expect(root.style.getPropertyValue('--bg')).toBe('')
})
