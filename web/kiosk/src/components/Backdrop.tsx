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
  return (
    <div data-testid="backdrop" aria-hidden="true" className="pointer-events-none fixed inset-0">
      {url && <Layers url={url} picture={picture} />}
    </div>
  )
}

/** Exists only while there is a backdrop, so it starts with the first one. */
function Layers({ url, picture }: { url: string; picture: string }) {
  const { layers, drop } = useCrossFade(picture, url, '')
  if (layers.length === 0) return null
  return (
    <>
      {layers.map((layer) => (
        <img
          key={layer.id}
          src={layer.url}
          alt=""
          onError={drop}
          className={`absolute inset-0 size-full object-cover ${layer.fades ? 'motion-safe:animate-fade-in' : ''}`}
        />
      ))}
      <div data-testid="veil" className="absolute inset-0 bg-black/45" />
    </>
  )
}
