/** A disc drawn in the theme's colours; turns slowly unless the viewer
 * prefers reduced motion (only `transform` is animated). */
export function Disc() {
  return (
    <svg
      role="img"
      aria-label="Disc"
      viewBox="0 0 100 100"
      className="size-[38vmin] text-kiosk-text motion-safe:animate-disc portrait:size-[46vmin]"
    >
      <circle cx="50" cy="50" r="48" fill="currentColor" fillOpacity="0.08" />
      <circle cx="50" cy="50" r="48" fill="none" stroke="currentColor" strokeOpacity="0.25" />
      {[40, 33, 26].map((r) => (
        <circle key={r} cx="50" cy="50" r={r} fill="none" stroke="currentColor" strokeOpacity="0.1" />
      ))}
      {/* A highlight, so the turning can be seen. */}
      <path d="M50 4 A46 46 0 0 1 86 22" fill="none" stroke="currentColor" strokeOpacity="0.35" strokeWidth="2" strokeLinecap="round" />
      <circle cx="50" cy="50" r="15" fill="rgb(var(--accent))" fillOpacity="0.35" />
      <circle cx="50" cy="50" r="3" fill="rgb(var(--bg))" />
    </svg>
  )
}
