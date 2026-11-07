import { act, renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { reloadForBuild } from './build'
import { FakeEventSource, player } from './test/fixtures'
import { usePlayer } from './usePlayer'

vi.mock('./build', () => ({ reloadForBuild: vi.fn() }))

beforeEach(() => {
  FakeEventSource.instances = []
  vi.stubGlobal('EventSource', FakeEventSource)
})

afterEach(() => {
  vi.unstubAllGlobals()
})

function stubFetch(serverDate: string | null) {
  const headers = new Headers(serverDate ? { Date: serverDate } : {})
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => new Response(JSON.stringify(player()), { status: 200, headers })),
  )
}

test('opens the live update stream', () => {
  stubFetch(null)
  renderHook(() => usePlayer())
  expect(FakeEventSource.latest().url).toBe('/api/v1/player/events')
})

test('applies the first snapshot from the stream', async () => {
  stubFetch(null)
  const { result } = renderHook(() => usePlayer())
  act(() => {
    FakeEventSource.latest().open()
    FakeEventSource.latest().send(player({ track: { title: 'Live Song', artists: [], album: null } }))
  })
  await waitFor(() => expect(result.current.player?.track?.title).toBe('Live Song'))
  expect(result.current.connection).toBe('live')
})

test('marks the connection as lost on error', async () => {
  stubFetch(null)
  const { result } = renderHook(() => usePlayer())
  act(() => {
    FakeEventSource.latest().open()
    FakeEventSource.latest().fail()
  })
  expect(result.current.connection).toBe('lost')
})

test('applies the current snapshot after reconnecting', async () => {
  stubFetch(null)
  const { result } = renderHook(() => usePlayer())
  const source = FakeEventSource.latest()
  act(() => {
    source.open()
    source.send(player({ track: { title: 'Before', artists: [], album: null } }))
    source.fail()
  })
  expect(result.current.connection).toBe('lost')

  // EventSource reconnects by itself; the server sends the current snapshot first.
  act(() => {
    source.open()
    source.send(player({ track: { title: 'After', artists: [], album: null } }))
  })
  expect(result.current.connection).toBe('live')
  expect(result.current.player?.track?.title).toBe('After')
})

test('corrects for the server clock', async () => {
  // The server is 10 seconds ahead of this device.
  const serverNow = Date.now() + 10_000
  stubFetch(new Date(serverNow).toUTCString())
  const { result } = renderHook(() => usePlayer())
  await waitFor(() => expect(Math.abs(result.current.clockOffsetMs - 10_000)).toBeLessThan(1500))
})

test('closes the stream when unmounted', () => {
  stubFetch(null)
  const { unmount } = renderHook(() => usePlayer())
  unmount()
  expect(FakeEventSource.latest().closed).toBe(true)
})

test('passes the announced build on, to reload after an update', () => {
  stubFetch(null)
  renderHook(() => usePlayer())
  act(() => FakeEventSource.latest().announce('build', 'd4e5f6'))
  expect(reloadForBuild).toHaveBeenCalledWith('d4e5f6')
})
