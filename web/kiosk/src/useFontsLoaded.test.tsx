// Spec: kiosk-display, "Typeface", "Measured after loading".
import { act, renderHook } from '@testing-library/react'
import { afterEach, expect, test } from 'vitest'
import { useFontsLoaded } from './useFontsLoaded'

const original = Object.getOwnPropertyDescriptor(document, 'fonts')
afterEach(() => {
  if (original) Object.defineProperty(document, 'fonts', original)
  else Reflect.deleteProperty(document, 'fonts')
})

test('counts up each time the page has finished loading fonts', () => {
  const fonts = new EventTarget()
  Object.defineProperty(document, 'fonts', { configurable: true, value: fonts })
  const { result } = renderHook(() => useFontsLoaded())
  const first = result.current
  act(() => {
    fonts.dispatchEvent(new Event('loadingdone'))
  })
  expect(result.current).toBe(first + 1)
})

test('counts the fonts that finished loading before it listened', async () => {
  // Already loaded: no "loadingdone" will come, but "ready" resolves.
  const fonts = Object.assign(new EventTarget(), { ready: Promise.resolve() })
  Object.defineProperty(document, 'fonts', { configurable: true, value: fonts })
  const { result } = renderHook(() => useFontsLoaded())
  await act(async () => {
    await fonts.ready
  })
  expect(result.current).toBe(1)
})

test('stays put without a font loading API', () => {
  Object.defineProperty(document, 'fonts', { configurable: true, value: undefined })
  const { result } = renderHook(() => useFontsLoaded())
  expect(result.current).toBe(0)
})
