// Spec: kiosk-display, end to end: the real backend with fake scenarios.
// Assertions check the layout; screenshots are saved for review in
// test-results/screens/ (no pixel comparison: fonts render differently per OS).
import { type Page, expect, test } from '@playwright/test'
import { type Scenario, portFor } from './scenarios.ts'

async function open(page: Page, scenario: Scenario) {
  await page.goto(`http://127.0.0.1:${portFor(scenario)}/`)
  await page.evaluate(() => document.fonts.ready)
}

async function screenshot(page: Page, scenario: Scenario) {
  const project = test.info().project.name
  await page.screenshot({ path: `test-results/screens/${project}-${scenario}.png` })
}

const portrait = () => test.info().project.name === 'portrait'

async function box(page: Page, selector: Parameters<Page['getByRole']>) {
  const element = page.getByRole(...selector)
  await expect(element).toBeVisible()
  const bounds = await element.boundingBox()
  if (!bounds) throw new Error('element has no box')
  return bounds
}

test('playing with queue', async ({ page }) => {
  await open(page, 'playing-with-queue')
  const art = await box(page, ['img', { name: 'Example Album' }])
  const title = await box(page, ['heading', { name: 'Example Song' }])
  if (portrait()) {
    expect(art.y + art.height).toBeLessThanOrEqual(title.y)
  } else {
    expect(art.x + art.width).toBeLessThanOrEqual(title.x)
  }
  // The generated cover actually loaded.
  const loaded = await page
    .getByRole('img', { name: 'Example Album' })
    .evaluate((img: HTMLImageElement) => img.complete && img.naturalWidth > 0)
  expect(loaded).toBe(true)

  await expect(page.getByRole('progressbar')).toBeVisible()
  await expect(page.getByText('Qobuz', { exact: true })).toBeVisible()
  await expect(page.getByText('FLAC 16/44.1 · CD quality')).toBeVisible()
  const upNext = page.getByRole('list', { name: 'Up next' }).getByRole('listitem')
  // Four entries in landscape; in portrait the fourth is hidden.
  await expect(upNext).toHaveCount(portrait() ? 3 : 4)
  await screenshot(page, 'playing-with-queue')
})

test('cast session without queue', async ({ page }) => {
  await open(page, 'cast-without-queue')
  await expect(page.getByRole('heading', { name: 'Cast Song' })).toBeVisible()
  await expect(page.getByText('Spotify', { exact: true })).toBeVisible()
  await expect(page.getByText('160 kbps')).toBeVisible()
  await expect(page.getByRole('list', { name: 'Up next' })).toHaveCount(0)
  await screenshot(page, 'cast-without-queue')
})

test('long title stays within two lines', async ({ page }) => {
  await open(page, 'long-title')
  const heading = page.getByRole('heading', { level: 1 })
  await expect(heading).toBeVisible()
  const lines = await heading.evaluate((element) => {
    const lineHeight = Number.parseFloat(getComputedStyle(element).lineHeight)
    return element.getBoundingClientRect().height / lineHeight
  })
  expect(lines).toBeLessThanOrEqual(2.05)
  await screenshot(page, 'long-title')
})

test('missing artwork shows initials', async ({ page }) => {
  await open(page, 'no-artwork')
  await expect(page.getByRole('img', { name: 'Example Album' })).toHaveText('EA')
  await screenshot(page, 'no-artwork')
})

test('hi-res track shows the HI-RES badge', async ({ page }) => {
  await open(page, 'hi-res')
  await expect(page.getByText('HI-RES')).toBeVisible()
  await expect(page.getByText('FLAC 24/96')).toBeVisible()
  await screenshot(page, 'hi-res')
})

test('paused shows the pause symbol', async ({ page }) => {
  await open(page, 'paused')
  const symbol = await box(page, ['status', { name: 'Paused' }])
  const art = await box(page, ['img', { name: 'Example Album' }])
  // The symbol is centred on the artwork, not on the screen.
  expect(Math.abs(symbol.x + symbol.width / 2 - (art.x + art.width / 2))).toBeLessThan(2)
  expect(Math.abs(symbol.y + symbol.height / 2 - (art.y + art.height / 2))).toBeLessThan(2)
  await expect(page.getByRole('heading', { name: 'Example Song' })).toBeVisible()
  await screenshot(page, 'paused')
})

test('stopped shows the idle screen', async ({ page }) => {
  await open(page, 'stopped')
  await expect(page.getByText('Example Player · nothing playing')).toBeVisible()
  await expect(page.getByRole('timer')).toBeVisible()
  await screenshot(page, 'stopped')
})

test('offline keeps the screen and shows the indicator', async ({ page }) => {
  await open(page, 'offline')
  await expect(page.getByRole('alert')).toHaveText('Player offline')
  await expect(page.getByRole('heading', { name: 'Example Song' })).toBeVisible()
  await screenshot(page, 'offline')
})

test('the API sends a Date header for the clock offset', async ({ request }) => {
  const response = await request.get(`http://127.0.0.1:${portFor('playing-with-queue')}/api/v1/player`)
  expect(response.headers()['date']).toBeTruthy()
})
