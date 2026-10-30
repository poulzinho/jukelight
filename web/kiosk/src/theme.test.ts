// @vitest-environment node
import { readFileSync } from 'node:fs'
import { expect, test } from 'vitest'

const css = readFileSync(new URL('./index.css', import.meta.url), 'utf8')

test.each([
  ['--text-title', '7.2vmin'],
  ['--text-artist', '4.4vmin'],
  ['--text-album', '3.2vmin'],
  ['--text-time', '2.4vmin'],
  ['--text-badge', '2.2vmin'],
  ['--text-next-title', '2.6vmin'],
  ['--text-next-artist', '2.2vmin'],
  ['--text-clock', '16vmin'],
  ['--spacing-page-y', '8vmin'],
  ['--spacing-page-x', '7vw'],
  ['--spacing-columns', '6vw'],
])('theme token %s is %s', (token, value) => {
  expect(css).toMatch(new RegExp(`${token}:\\s*${value.replace('.', '\\.')};`))
})

test('colours come from per-track variables', () => {
  for (const variable of ['--bg', '--accent', '--text']) {
    expect(css).toContain(`var(${variable})`)
  }
})

test('the font is the bundled Inter', () => {
  expect(css).toContain("@import '@fontsource-variable/inter'")
  expect(css).toMatch(/--font-sans:\s*'Inter Variable'/)
})