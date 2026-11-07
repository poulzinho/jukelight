import { useCrossFade } from '../useCrossFade'

interface Props {
  url: string | null
  /** Names the picture: the backdrop stays while it is the same. */
  picture: string
}

/**
 * The cover's tiny version stretched over the whole screen: stretching
 * blurs it, so no CSS filter is needed (expensive at 4K on an integrated
 * GPU). A dark veil keeps the text readable. Without an image, or when it
 * fails, the plain background colour shows.
 */
export function Backdrop({ url, picture }: Props) {
  const { layers, drop } = useCrossFade(picture, url, '', 'drop')
  return (
    <div data-testid="backdrop" aria-hidden="true" className="pointer-events-none fixed inset-0">
      {layers.map((layer) => (
        <img
          key={layer.id}
          src={layer.url}
          alt=""
          onError={drop}
          className={`absolute inset-0 size-full object-cover ${layer.fades ? 'motion-safe:animate-fade-in' : ''}`}
        />
      ))}
      {layers.length > 0 && <div data-testid="veil" className="absolute inset-0 bg-black/45" />}
    </div>
  )
}
