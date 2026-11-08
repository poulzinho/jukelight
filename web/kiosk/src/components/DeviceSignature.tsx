import { RadioReceiver } from 'lucide-react'
import type { Player } from '../api'

/** Makers the kiosk can name without their logo. */
const MAKERS: Record<string, string> = { bluesound: 'Bluesound' }

/**
 * The player that plays, quietly at the bottom of every screen: its maker's
 * logo (when the user supplied one), or a streamer icon and the maker's
 * name, then the player's name. Centred in portrait, in the bottom-right
 * corner in landscape.
 */
export function DeviceSignature({ device }: { device: Player['device'] }) {
  const maker = device.brand ? MAKERS[device.brand] : undefined
  const mark = device.logoUrl ? (
    // Decorative: the player's name follows.
    <img src={device.logoUrl} alt="" aria-hidden="true" className="block h-[1.6vmin] w-auto opacity-80" />
  ) : maker ? (
    <>
      <RadioReceiver aria-hidden="true" className="size-[2.2vmin] shrink-0 opacity-80" />
      <span>{maker}</span>
    </>
  ) : null
  return (
    <footer
      data-testid="device-signature"
      className="pointer-events-none fixed inset-x-0 bottom-[2.6vh] flex items-center justify-center gap-[1.2vmin] text-[1.8vmin] font-medium text-kiosk-text/50 landscape:bottom-[3vmin] landscape:left-auto landscape:right-page-x"
    >
      {mark}
      {mark && (
        <span aria-hidden="true" className="text-kiosk-text/30">
          ·
        </span>
      )}
      <span>{device.name}</span>
    </footer>
  )
}
