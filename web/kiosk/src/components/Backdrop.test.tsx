// Spec: kiosk-display, "Backdrop".
import { fireEvent, render, screen } from '@testing-library/react'
import { expect, test } from 'vitest'
import { Kiosk } from '../Kiosk'
import { player } from '../test/fixtures'

function show(overrides: Parameters<typeof player>[0] = {}) {
  return render(<Kiosk player={player(overrides)} connection="live" clockOffsetMs={0} />)
}

test('track with a backdrop: the image covers the screen behind the veil', () => {
  show()
  const backdrop = screen.getByTestId('backdrop')
  const image = backdrop.querySelector('img')
  expect(image).toHaveAttribute('src', '/api/v1/art/0123456789abcdef?size=backdrop')
  expect(image?.className).toContain('object-cover')
  expect(screen.getByTestId('veil')).toBeInTheDocument()
  // Blurred by stretching the tiny image, never by a CSS filter.
  expect(backdrop.outerHTML).not.toMatch(/blur|filter/)
})

test('no backdrop: the plain background colour', () => {
  show({ backdropUrl: null })
  expect(screen.getByTestId('backdrop').querySelector('img')).toBeNull()
})

test('a backdrop that fails to load leaves the plain background colour', () => {
  show()
  const image = screen.getByTestId('backdrop').querySelector('img')
  if (!image) throw new Error('no backdrop image')
  fireEvent.error(image)
  expect(screen.getByTestId('backdrop').querySelector('img')).toBeNull()
})
