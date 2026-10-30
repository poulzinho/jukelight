import { render, screen } from '@testing-library/react'
import { afterEach, expect, test, vi } from 'vitest'
import App from './App'
import { FakeEventSource, player } from './test/fixtures'

afterEach(() => {
  vi.unstubAllGlobals()
})

test('connects the kiosk to the live player', async () => {
  vi.stubGlobal('EventSource', FakeEventSource)
  vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify(player()))))
  render(<App />)
  expect(await screen.findByRole('heading', { name: 'Example Song' })).toBeInTheDocument()
})
