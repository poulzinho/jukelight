/**
 * A dark layer over the whole screen: dims the content cheaply (opacity, not
 * a filter). Render it inside the positioned page container.
 */
export function Dim() {
  return (
    <div
      data-testid="dim"
      className="pointer-events-none absolute inset-0 z-10 bg-black/55 motion-safe:animate-fade-in"
    />
  )
}

/**
 * A large pause symbol at full brightness, readable from across the room.
 * Render it inside the positioned artwork container: it sits above the dim
 * layer and centred on the artwork.
 */
export function PauseSymbol() {
  return (
    <div
      role="status"
      aria-label="Paused"
      className="pointer-events-none absolute inset-0 z-20 flex items-center justify-center gap-[5vmin] motion-safe:animate-fade-in"
    >
      <span className="h-[26vmin] w-[7vmin] rounded-[1.4vmin] bg-white/90 shadow-[0_1vmin_5vmin_rgb(0_0_0/0.55)]" />
      <span className="h-[26vmin] w-[7vmin] rounded-[1.4vmin] bg-white/90 shadow-[0_1vmin_5vmin_rgb(0_0_0/0.55)]" />
    </div>
  )
}
