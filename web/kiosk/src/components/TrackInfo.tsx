import type { Player } from '../api'

export function TrackInfo({ track }: { track: NonNullable<Player['track']> }) {
  return (
    <div className="flex min-w-0 flex-col gap-[1.6vmin]">
      {/* In portrait each line keeps its room, even when empty, so nothing below moves. */}
      <h1 className="line-clamp-2 text-title font-bold tracking-tight portrait:min-h-[2lh]">{track.title}</h1>
      {track.artists.length > 0 ? (
        <p className="truncate text-artist font-medium text-kiosk-text/85">{track.artists.join(', ')}</p>
      ) : (
        <p aria-hidden="true" className="hidden text-artist portrait:block portrait:min-h-[1lh]" />
      )}
      {track.album ? (
        <p className="truncate text-album text-kiosk-text/60">{track.album}</p>
      ) : (
        <p aria-hidden="true" className="hidden text-album portrait:block portrait:min-h-[1lh]" />
      )}
    </div>
  )
}
