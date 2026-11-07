import { initials } from '../time'
import { useCrossFade } from '../useCrossFade'

interface Props {
  url: string | null
  /** Names the picture: the cover stays while it is the same. */
  picture: string
  /** Album, or artists when there is no album: names the image. */
  label: string
}

const frame =
  'aspect-square w-[min(76vmin,44vw)] rounded-[1.2vmin] shadow-[0_3vmin_8vmin_rgb(0_0_0/0.55)] portrait:w-[min(86vw,42vh)]'

/** The track's cover, cross-faded when the picture changes. */
export function Artwork({ url, picture, label }: Props) {
  const { layers } = useCrossFade(picture, url, label, 'show')
  if (!url || layers.length === 0) {
    return (
      <div
        role="img"
        aria-label={label}
        className={`${frame} grid place-items-center bg-white/5 text-[16vmin] font-bold text-kiosk-text/35`}
      >
        {initials(label)}
      </div>
    )
  }
  return (
    <div className={`${frame} relative`}>
      {layers.map((layer, index) => (
        <img
          key={layer.id}
          src={layer.url}
          // Only the image on top names the cover.
          alt={index === layers.length - 1 ? layer.label : ''}
          className={`absolute inset-0 size-full rounded-[1.2vmin] object-cover ${layer.fades ? 'motion-safe:animate-fade-in' : ''}`}
        />
      ))}
    </div>
  )
}
