import { useEffect, useState } from 'react'
import { EVENTS_URL, PLAYER_URL, type Player } from './api'

export type Connection = 'connecting' | 'live' | 'lost'

export interface PlayerState {
  player: Player | null
  connection: Connection
  /** Server clock minus this device's clock, in milliseconds. */
  clockOffsetMs: number
}

/**
 * The latest player snapshot, kept current over Server-Sent Events.
 *
 * EventSource reconnects by itself; after each reconnect the server sends
 * the current snapshot first, so no separate refetch is needed. The first
 * plain request measures the server clock, so progress stays right on a
 * device whose clock is off.
 */
export function usePlayer(): PlayerState {
  const [player, setPlayer] = useState<Player | null>(null)
  const [connection, setConnection] = useState<Connection>('connecting')
  const [clockOffsetMs, setClockOffsetMs] = useState(0)

  useEffect(() => {
    let cancelled = false
    const requested = Date.now()
    fetch(PLAYER_URL)
      .then(async (response) => {
        const received = Date.now()
        const serverDate = response.headers.get('Date')
        const snapshot = (await response.json()) as Player
        if (cancelled) return
        if (serverDate) {
          // The Date header has whole seconds; aim at the middle of the request.
          setClockOffsetMs(Date.parse(serverDate) + 500 - (requested + received) / 2)
        }
        setPlayer((current) => current ?? snapshot)
      })
      .catch(() => {
        // The live stream reports connection problems.
      })

    const source = new EventSource(EVENTS_URL)
    source.onopen = () => setConnection('live')
    source.onmessage = (event: MessageEvent<string>) => {
      setPlayer(JSON.parse(event.data) as Player)
      setConnection('live')
    }
    source.onerror = () => setConnection('lost')

    return () => {
      cancelled = true
      source.close()
    }
  }, [])

  return { player, connection, clockOffsetMs }
}
