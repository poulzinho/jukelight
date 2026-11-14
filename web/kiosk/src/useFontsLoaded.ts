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
    const loaded = () => setLoads((count) => count + 1)
    fonts.addEventListener('loadingdone', loaded)
    return () => fonts.removeEventListener('loadingdone', loaded)
  }, [])
  return loads
}
