import { type ReactNode, useEffect, useState } from 'react'

/** How long the old text takes to fade out (the fade-out animation). */
const FADE_OUT_MS = 300

interface Props {
  /** Names the content: a new name swaps it. */
  name: string
  children: ReactNode
}

/**
 * Text that changes gently: when `name` changes, the old content fades out,
 * then the new content fades in while rising slightly. With reduced motion,
 * the new content replaces the old at once.
 */
export function TextSwap({ name, children }: Props) {
  const [shown, setShown] = useState({ name, children })
  const changing = shown.name !== name

  useEffect(() => {
    if (!changing) return
    const reduced = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false
    const timer = setTimeout(() => setShown({ name, children }), reduced ? 0 : FADE_OUT_MS)
    return () => clearTimeout(timer)
  }, [changing, name, children])

  if (changing) {
    return <div className="motion-safe:animate-fade-out">{shown.children}</div>
  }
  return (
    <div key={name} className="motion-safe:animate-rise-in">
      {children}
    </div>
  )
}
