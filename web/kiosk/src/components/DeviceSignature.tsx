import type { Player } from '../api'

/**
 * The player that plays: its maker's logo (when the user supplied one) and
 * its name, quietly at the bottom of every screen. Centred in portrait, in
 * the bottom-right corner in landscape.
 */
export function DeviceSignature({ device }: { device: Player['device'] }) {
  return (
    <footer
      data-testid="device-signature"
      className="pointer-events-none fixed inset-x-0 bottom-[2.6vh] flex items-center justify-center gap-[1.2vmin] text-[1.8vmin] font-medium text-kiosk-text/50 landscape:bottom-[3vmin] landscape:left-auto landscape:right-page-x"
    >
      {device.logoUrl && (
        // Decorative: the player's name follows.
        <img src={device.logoUrl} alt="" aria-hidden="true" className="block h-[1.6vmin] w-auto opacity-80" />
      )}
      {device.logoUrl && (
        <span aria-hidden="true" className="text-kiosk-text/30">
          ·
        </span>
      )}
      <span>{device.name}</span>
    </footer>
  )
}
