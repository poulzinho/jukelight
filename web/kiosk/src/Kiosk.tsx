import type { Player } from './api'
import { Artwork } from './components/Artwork'
import { Badges } from './components/Badges'
import { ConnectionIndicator } from './components/ConnectionIndicator'
import { IdleScreen } from './components/IdleScreen'
import { Dim, PauseSymbol } from './components/PauseOverlay'
import { Progress } from './components/Progress'
import { TrackInfo } from './components/TrackInfo'
import { UpNext } from './components/UpNext'
import type { Connection } from './usePlayer'

interface Props {
  player: Player | null
  connection: Connection
  clockOffsetMs: number
}

/** The kiosk screen for one snapshot. */
export function Kiosk({ player, connection, clockOffsetMs }: Props) {
  const indicator =
    connection === 'lost' ? 'Reconnecting…' : player && !player.online ? 'Player offline' : null

  return (
    <>
      {player?.track && player.state !== 'stopped' ? (
        <PlayingScreen player={player} track={player.track} clockOffsetMs={clockOffsetMs} />
      ) : (
        <IdleScreen deviceName={player?.deviceName || null} />
      )}
      {indicator && <ConnectionIndicator message={indicator} />}
    </>
  )
}

interface PlayingProps {
  player: Player
  track: NonNullable<Player['track']>
  clockOffsetMs: number
}

function PlayingScreen({ player, track, clockOffsetMs }: PlayingProps) {
  const paused = player.state === 'paused'
  return (
    <>
      <main
        // A new track remounts the content, which fades it in.
        key={`${track.title}|${track.album ?? ''}`}
        className="relative grid h-full grid-cols-[auto_1fr] items-center gap-x-columns px-page-x py-page-y motion-safe:animate-fade-in portrait:grid-cols-1 portrait:content-center portrait:justify-items-center portrait:gap-y-stack portrait:py-[6vh]"
      >
        <div className="relative">
          <Artwork url={player.artUrl} label={track.album ?? track.artists.join(', ')} />
          {paused && <PauseSymbol />}
        </div>
        <section className="flex min-w-0 flex-col portrait:w-[min(86vw,48vh)]">
          <TrackInfo track={track} />
          {player.position && (
            <Progress position={player.position} state={player.state} clockOffsetMs={clockOffsetMs} />
          )}
          <Badges player={player} />
          {player.queue && <UpNext queue={player.queue} />}
        </section>
        {paused && <Dim />}
      </main>
    </>
  )
}
