import { useEffect } from 'react'
import type { Player } from './api'

/**
 * Sets the page's --bg and --accent to the album colours, or removes them
 * so the defaults in index.css apply. Colours switch instantly: the kiosk
 * animates only opacity and transform.
 */
export function useAlbumColours(palette: Player['palette']) {
  const background = palette?.background.join(' ')
  const accent = palette?.accent.join(' ')

  useEffect(() => {
    const style = document.documentElement.style
    if (background && accent) {
      style.setProperty('--bg', background)
      style.setProperty('--accent', accent)
    } else {
      style.removeProperty('--bg')
      style.removeProperty('--accent')
    }
  }, [background, accent])
}
