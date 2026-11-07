import { useEffect, useState } from 'react'
import { loadImage } from './loadImage'

/** One image in a cross-fade; the last one is on top. */
export interface Layer {
  id: number
  url: string
  /** What the image shows, for its text alternative. */
  label: string
  /** Fades in on mount; the first image of a screen appears at once. */
  fades: boolean
}

interface State {
  /** The picture on screen; `null` before the first one. */
  picture: string | null
  layers: Layer[]
  next: number
}

/** How long a new image takes to fade in (the fade-in animation). */
const FADE_MS = 600

/**
 * The images to stack for a cover that cross-fades when the picture
 * changes. `picture` names the picture (for example album and artists): as
 * long as it stays the same, so does the image, even if its URL changes. A
 * new picture is loaded first and then fades in over the old one, which goes
 * once the fade has finished, so there's never a blank frame. `drop` removes
 * the current image, for one that failed to load.
 */
export function useCrossFade(
  picture: string | null,
  url: string | null,
  label: string,
  onFailure: 'show' | 'drop',
): { layers: Layer[]; drop: () => void } {
  const [state, setState] = useState<State>(() =>
    url
      ? { picture, layers: [{ id: 0, url, label, fades: false }], next: 1 }
      : { picture: null, layers: [], next: 0 },
  )

  // Without an image nothing shows; the first image appears at once.
  let current = state
  if (!url && (state.layers.length > 0 || state.picture !== null)) {
    current = { ...state, picture: null, layers: [] }
    setState(current)
  } else if (url && state.picture === null && state.layers.length === 0) {
    current = { picture, layers: [{ id: state.next, url, label, fades: false }], next: state.next + 1 }
    setState(current)
  }

  const shown = current.picture
  useEffect(() => {
    if (!url || shown === null || picture === shown) return
    let cancelled = false
    let cleanup: ReturnType<typeof setTimeout> | undefined
    const show = () => {
      if (cancelled) return
      setState((s) => ({
        picture,
        layers: [...s.layers.slice(-1), { id: s.next, url, label, fades: true }],
        next: s.next + 1,
      }))
      // The old image goes once the new one has faded in.
      cleanup = setTimeout(() => setState((s) => ({ ...s, layers: s.layers.slice(-1) })), FADE_MS)
    }
    loadImage(url).then(show, () => {
      if (onFailure === 'show') show()
      else if (!cancelled) setState((s) => ({ ...s, picture, layers: [] }))
    })
    return () => {
      cancelled = true
      clearTimeout(cleanup)
    }
  }, [picture, url, label, shown, onFailure])

  return { layers: current.layers, drop: () => setState((s) => ({ ...s, layers: [] })) }
}
