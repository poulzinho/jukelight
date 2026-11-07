// Spec: kiosk-display, "Reload after an update".
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { reloadForBuild } from './build'

function pageWithBuild(build: string | null) {
  document.head.querySelector('meta[name="jukelight-build"]')?.remove()
  if (build === null) return
  const meta = document.createElement('meta')
  meta.name = 'jukelight-build'
  meta.content = build
  document.head.append(meta)
}

beforeEach(() => sessionStorage.clear())
afterEach(() => pageWithBuild(null))

test('new build: reloads', () => {
  pageWithBuild('a1b2c3')
  const reload = vi.fn()
  reloadForBuild('d4e5f6', reload)
  expect(reload).toHaveBeenCalledOnce()
})

test('same build: no reload', () => {
  pageWithBuild('a1b2c3')
  const reload = vi.fn()
  reloadForBuild('a1b2c3', reload)
  expect(reload).not.toHaveBeenCalled()
})

test('still different after reloading: no second reload for the same build', () => {
  pageWithBuild('a1b2c3')
  const reload = vi.fn()
  reloadForBuild('d4e5f6', reload)
  // The reloaded page still carries the old build, for example from a stale cache.
  reloadForBuild('d4e5f6', reload)
  expect(reload).toHaveBeenCalledOnce()
  // A later, different build still reloads.
  reloadForBuild('a7b8c9', reload)
  expect(reload).toHaveBeenCalledTimes(2)
})

test('no build in the page: no reload', () => {
  pageWithBuild(null)
  const reload = vi.fn()
  reloadForBuild('d4e5f6', reload)
  expect(reload).not.toHaveBeenCalled()
})
