/** Where the kiosk notes the build it last reloaded for. */
const RELOADED_FOR = 'jukelight-reloaded-for'

/**
 * The build of this page, as the backend wrote it into the page when it
 * served it; none on the development server.
 */
function pageBuild(): string | null {
  return (
    document.querySelector<HTMLMetaElement>('meta[name="jukelight-build"]')?.content || null
  )
}

/**
 * Reloads the page when the backend announces a different build than the
 * page's, so the kiosk runs the new version after an update. At most once
 * per announced build: if the reloaded page still carries the old build
 * (a stale cache), it doesn't reload again in a loop.
 */
export function reloadForBuild(announced: string, reload = () => location.reload()): void {
  const own = pageBuild()
  if (!own || !announced || own === announced) return
  if (sessionStorage.getItem(RELOADED_FOR) === announced) return
  sessionStorage.setItem(RELOADED_FOR, announced)
  reload()
}
