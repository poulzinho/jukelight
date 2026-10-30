import { initials } from '../time'

interface Props {
  url: string | null
  /** Album, or artists when there is no album: names the image. */
  label: string
}

const frame =
  'aspect-square w-[min(76vmin,44vw)] rounded-[1.2vmin] shadow-[0_3vmin_8vmin_rgb(0_0_0/0.55)] portrait:w-[min(86vw,48vh)]'

export function Artwork({ url, label }: Props) {
  if (!url) {
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
  return <img src={url} alt={label} className={`${frame} object-cover`} />
}
