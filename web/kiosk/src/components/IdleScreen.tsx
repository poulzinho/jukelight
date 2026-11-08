import { Clock } from './Clock'

/** Nothing plays: the time. The player's name is in the signature, once
 * the first snapshot has named the player. */
export function IdleScreen() {
  return (
    <main className="grid h-full place-items-center text-center motion-safe:animate-fade-in">
      <div>
        <Clock />
        <p className="mt-[1vmin] text-idle text-kiosk-text/55">
          nothing playing
        </p>
      </div>
    </main>
  )
}
