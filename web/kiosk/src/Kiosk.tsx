import type { Player } from './api'
import { Artwork } from './components/Artwork'
import { Backdrop } from './components/Backdrop'
import { ConnectionIndicator } from './components/ConnectionIndicator'
import { DeviceSignature } from './components/DeviceSignature'
import { IdleScreen } from './components/IdleScreen'
import { InputScreen } from './components/InputScreen'
import { Dim, PauseSymbol } from './components/PauseOverlay'
import { Progress, ProgressSlot } from './components/Progress'
import { SignalPath } from './components/SignalPath'
import { TextSwap } from './components/TextSwap'
import { TrackInfo } from './components/TrackInfo'
import { UpNext } from './components/UpNext'
import { useAlbumColours } from './useAlbumColours'
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
  const playing = Boolean(player?.track) && player?.state !== 'stopped'
  // The idle screen keeps the default colours.
  useAlbumColours(playing ? (player?.palette ?? null) : null)

  return (
    <>
      {player?.input && player.state !== 'stopped' ? (
        <InputScreen player={player} input={player.input} clockOffsetMs={clockOffsetMs} />
      ) : player?.track && playing ? (
        <PlayingScreen player={player} track={player.track} clockOffsetMs={clockOffsetMs} />
      ) : (
        <IdleScreen />
      )}
      {player && <DeviceSignature device={player.device} />}
      {indicator && <ConnectionIndicator message={indicator} />}
    </>
  )
}

interface PlayingProps {
  player: Player
  track: NonNullable<Player['track']>
  clockOffsetMs: number
}

/**
 * Names the cover's picture: the same album and artists keep the cover, even
 * though a player may give each song its own image URL. Without an album,
 * the URL decides.
 */
function pictureOf(player: Player, track: NonNullable<Player['track']>): string {
  return track.album ? `${track.album}|${track.artists.join(', ')}` : (player.artUrl ?? '')
}

function PlayingScreen({ player, track, clockOffsetMs }: PlayingProps) {
  const paused = player.state === 'paused'
  const picture = pictureOf(player, track)
  return (
    <>
      <Backdrop url={player.backdropUrl} picture={picture} />
      <main
        // In portrait every part has a fixed slot from the top, with "Up
        // next" at the bottom, so changing content never moves the others.
        className="relative grid h-full grid-cols-[auto_1fr] items-center gap-x-columns px-page-x py-page-y motion-safe:animate-fade-in portrait:grid-cols-1 portrait:grid-rows-[auto_1fr] portrait:items-start portrait:justify-items-center portrait:gap-y-stack portrait:pt-[5vh] portrait:pb-[7vh]"
      >
        <div className="relative">
          <Artwork
            url={player.artUrl}
            picture={picture}
            label={track.album ?? track.artists.join(', ')}
          />
          {paused && <PauseSymbol />}
        </div>
        <section className="flex min-w-0 flex-col portrait:h-full portrait:w-[min(86vw,42vh)]">
          {/* Only the track text changes for a new track: the old fades out, the new fades in. */}
          <TextSwap name={`${track.title}|${track.artists.join(', ')}|${track.album ?? ''}`}>
            <TrackInfo track={track} />
          </TextSwap>
          {player.position ? (
            <Progress position={player.position} state={player.state} clockOffsetMs={clockOffsetMs} />
          ) : (
            <ProgressSlot />
          )}
          <SignalPath player={player} />
          {player.queue && <UpNext queue={player.queue} />}
        </section>
        {paused && <Dim />}
      </main>
    </>
  )
}
