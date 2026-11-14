import { useEffect, useState } from 'react'

/**
 * Counts how many times the page has finished loading fonts, so text
 * measured with the page's font can be measured again once it is there.
 */
export function useFontsLoaded(): number {
  const [loads, setLoads] = useState(0)
  useEffect(() => {
    const fonts = document.fonts as FontFaceSet | undefined
    if (!fonts?.addEventListener) return
    let listening = true
    const loaded = () => {
      if (listening) setLoads((count) => count + 1)
    }
    fonts.addEventListener('loadingdone', loaded)
    // Fonts that finished before this listened send no "loadingdone":
    // "ready" resolves for them.
    void fonts.ready?.then(loaded)
    return () => {
      listening = false
      fonts.removeEventListener('loadingdone', loaded)
    }
  }, [])
  return loads
}
