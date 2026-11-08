/** A small corner note; the last screen stays visible behind it. */
export function ConnectionIndicator({ message }: { message: string }) {
  return (
    <div
      role="alert"
      className="fixed top-[2.4vmin] right-[2.4vmin] rounded-full bg-black/50 px-[1.6vmin] py-[0.8vmin] text-[2vmin] text-[rgb(255_200_120)] motion-safe:animate-fade-in"
    >
      {message}
    </div>
  )
}
