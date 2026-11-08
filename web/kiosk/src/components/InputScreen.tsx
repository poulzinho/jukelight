import type { Player } from '../api'
import { useNow } from '../useNow'
import { SourceIcon } from './Brand'
import { Clock } from './Clock'
import { Disc } from './Disc'

interface Props {
  player: Player
  input: NonNullable<Player['input']>
  clockOffsetMs: number
}

/** An audio input plays: nobody knows the track, so the screen shows the
 * time, a turning disc, the input's name and how long its signal plays. */
export function InputScreen({ player, input, clockOffsetMs }: Props) {
  return (
    <main className="grid h-full grid-cols-[auto_1fr] items-center gap-x-columns px-page-x py-page-y motion-safe:animate-fade-in portrait:grid-cols-1 portrait:content-center portrait:justify-items-center portrait:gap-y-stack">
      <Disc />
      <section className="flex min-w-0 flex-col portrait:items-center portrait:text-center">
        <Clock />
        <h1 className="mt-[2vmin] text-title font-bold tracking-tight text-kiosk-text">{input.name}</h1>
        <p className="mt-[2vmin] flex items-center gap-[0.45em] text-signal font-semibold text-kiosk-text/80">
          <SourceIcon source={{ brand: null, name: player.session.label }} />
          {player.session.label}
        </p>
        <PlayingFor startedAt={input.startedAt} clockOffsetMs={clockOffsetMs} />
      </section>
    </main>
  )
}

function PlayingFor({ startedAt, clockOffsetMs }: { startedAt: number | null; clockOffsetMs: number }) {
  const now = useNow(30_000) + clockOffsetMs
  const text =
    startedAt === null
      ? 'waiting for signal'
      : `for ${Math.max(0, Math.floor((now - startedAt) / 60_000))} min`
  return <p className="mt-[2vmin] text-artist text-kiosk-text/60 tabular-nums">{text}</p>
}
