import type { Player } from '../api'

/** A playing track with everything known. Override fields per test. */
export function player(overrides: Partial<Player> = {}): Player {
  return {
    online: true,
    deviceName: 'Example Player',
    state: 'playing',
    session: { label: 'Qobuz', available: true },
    track: { title: 'Example Song', artists: ['Example Artist'], album: 'Example Album' },
    artUrl: '/api/v1/art/0123456789abcdef',
    position: { elapsed: 60, duration: 200, observedAt: 1_000_000 },
    quality: { label: 'FLAC 16/44.1', detail: 'CD quality', hiRes: false },
    queue: null,
    palette: null,
    input: null,
    ...overrides,
  }
}

/** An EventSource the test drives by hand. */
export class FakeEventSource {
  static instances: FakeEventSource[] = []
  onopen: (() => void) | null = null
  onmessage: ((event: { data: string }) => void) | null = null
  onerror: (() => void) | null = null
  closed = false
  url: string
  listeners = new Map<string, ((event: MessageEvent<string>) => void)[]>()

  constructor(url: string) {
    this.url = url
    FakeEventSource.instances.push(this)
  }

  close() {
    this.closed = true
  }

  addEventListener(name: string, listener: (event: MessageEvent<string>) => void) {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }

  /** Sends a named event, such as `build`. */
  announce(name: string, data: string) {
    for (const listener of this.listeners.get(name) ?? []) {
      listener(new MessageEvent(name, { data }))
    }
  }

  open() {
    this.onopen?.()
  }

  send(data: Player) {
    this.onmessage?.({ data: JSON.stringify(data) })
  }

  fail() {
    this.onerror?.()
  }

  static latest(): FakeEventSource {
    const source = FakeEventSource.instances.at(-1)
    if (!source) throw new Error('no EventSource was opened')
    return source
  }
}
