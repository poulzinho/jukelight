import { useEffect, useState } from 'react'
import { loadImage } from './loadImage'

/** One image in a cross-fade; the last one is on top. */
export interface Layer {
  id: number
  url: string
  /** What the image shows, for its text alternative. */
  label: string
  /** Fades in on mount; the first image appears at once. */
  fades: boolean
}

interface State {
  /** The picture the layers show. */
  picture: string
  layers: Layer[]
  next: number
  /** The URL that last failed, so it isn't tried again and again. */
  failed: string | null
}

/**
 * The images to stack for a cover that cross-fades when the picture
 * changes. Use it in a component that exists only while there is an image:
 * the first image appears at once.
 *
 * `picture` names the picture (for example album and artists): as long as it
 * stays the same, so does the image, even if its URL changes. A new picture
 * is loaded first and then fades in over the old one, which goes once the
 * fade has finished, so there's never a blank frame. An image that fails to
 * load leaves no layer (call `drop` for one that fails on screen); a new URL
 * is tried again.
 */
export function useCrossFade(
  picture: string,
  url: string,
  label: string,
  /** How long a new image takes to fade in: the length of its animation. */
  fadeMs: number,
): { layers: Layer[]; drop: () => void } {
  const [state, setState] = useState<State>(() => ({
    picture,
    layers: [{ id: 0, url, label, fades: false }],
    next: 1,
    failed: null,
  }))

  const samePicture = picture === state.picture
  const retry = state.layers.length === 0 && url !== state.failed
  useEffect(() => {
    if (samePicture && !retry) return
    let cancelled = false
    let cleanup: ReturnType<typeof setTimeout> | undefined
    loadImage(url).then(
      () => {
        if (cancelled) return
        setState((s) => ({
          picture,
          // Fades in only over an image that is already there.
          layers: [...s.layers.slice(-1), { id: s.next, url, label, fades: s.layers.length > 0 }],
          next: s.next + 1,
          failed: null,
        }))
        // The old image goes once the new one has faded in.
        cleanup = setTimeout(() => setState((s) => ({ ...s, layers: s.layers.slice(-1) })), fadeMs)
      },
      () => {
        if (!cancelled) setState((s) => ({ ...s, picture, layers: [], failed: url }))
      },
    )
    return () => {
      cancelled = true
      clearTimeout(cleanup)
    }
  }, [samePicture, retry, picture, url, label, fadeMs])

  const drop = () =>
    setState((s) => ({ ...s, layers: [], failed: s.layers.at(-1)?.url ?? s.failed }))
  return { layers: state.layers, drop }
}
