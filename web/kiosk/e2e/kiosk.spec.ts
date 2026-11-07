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

// Spec: kiosk-display, "Album colours".
async function colours(page: Page) {
  const background = await page.evaluate(() => getComputedStyle(document.body).backgroundColor)
  const fill = await page
    .getByRole('progressbar')
    .locator(':scope > div')
    .evaluate((element) => getComputedStyle(element).backgroundColor)
  return { background, fill }
}

test('album colours tint the page and the progress bar', async ({ page }) => {
  await open(page, 'album-colours')
  await expect(page.getByRole('heading', { name: 'Example Song' })).toBeVisible()
  expect(await colours(page)).toEqual({
    background: 'rgb(73, 51, 38)',
    fill: 'rgb(209, 147, 108)',
  })
  await screenshot(page, 'album-colours')
})

test('without album colours the default theme applies', async ({ page }) => {
  await open(page, 'playing-with-queue')
  await expect(page.getByRole('heading', { name: 'Example Song' })).toBeVisible()
  expect((await colours(page)).background).toBe('rgb(18, 18, 22)')
})

// Spec: kiosk-display, "Input screen".
test('optical input shows the input screen', async ({ page }) => {
  await open(page, 'optical-input')
  const disc = await box(page, ['img', { name: 'Disc' }])
  const name = await box(page, ['heading', { name: 'Example Input' }])
  if (portrait()) {
    expect(disc.y + disc.height).toBeLessThanOrEqual(name.y)
  } else {
    expect(disc.x + disc.width).toBeLessThanOrEqual(name.x)
  }
  await expect(page.getByRole('timer')).toBeVisible()
  await expect(page.getByText('Optical input', { exact: true })).toBeVisible()
  await expect(page.getByText('for 23 min')).toBeVisible()
  await expect(page.getByRole('progressbar')).toHaveCount(0)
  await expect(page.getByRole('list', { name: 'Up next' })).toHaveCount(0)
  // The project runs with reduced motion: the disc stands still.
  const animation = await page
    .getByRole('img', { name: 'Disc' })
    .evaluate((element) => getComputedStyle(element).animationName)
  expect(animation).toBe('none')
  await screenshot(page, 'optical-input')
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

// Spec: kiosk-display, "No translation prompt".
test('the page asks browsers not to translate it', async ({ page }) => {
  await open(page, 'playing-with-queue')
  const marks = await page.evaluate(() => ({
    translate: document.documentElement.getAttribute('translate'),
    meta: document.querySelector('meta[name="google"]')?.getAttribute('content') ?? null,
  }))
  expect(marks).toEqual({ translate: 'no', meta: 'notranslate' })
})

test('the API sends a Date header for the clock offset', async ({ request }) => {
  const response = await request.get(`http://127.0.0.1:${portFor('playing-with-queue')}/api/v1/player`)
  expect(response.headers()['date']).toBeTruthy()
})

// Spec: kiosk-display, "Reduced motion".
async function motion(page: Page) {
  await open(page, 'playing-with-queue')
  const animation = await page
    .getByRole('main')
    .evaluate((element) => getComputedStyle(element).animationName)
  const transition = await page
    .getByRole('progressbar')
    .locator('div')
    .evaluate((element) => getComputedStyle(element).transitionDuration)
  return { animation, transition }
}

test('reduced motion switches states without transitions', async ({ page }) => {
  // The project runs with reducedMotion: 'reduce'.
  expect(await motion(page)).toEqual({ animation: 'none', transition: '0s' })
})

test.describe('without reduced motion', () => {
  test.use({ reducedMotion: 'no-preference' })

  test('the disc turns', async ({ page }) => {
    await open(page, 'optical-input')
    const animation = await page
      .getByRole('img', { name: 'Disc' })
      .evaluate((element) => getComputedStyle(element).animationName)
    expect(animation).toBe('disc-turn')
  })

  test('states fade and progress moves smoothly', async ({ page }) => {
    expect(await motion(page)).toEqual({ animation: 'fade-in', transition: '0.3s' })
  })
})

// Spec: kiosk-display, "Steady portrait layout": the same parts in the same
// places, whatever the content.
test.describe('steady portrait layout', () => {
  test.skip(() => !portrait(), 'portrait only')

  /** Within a pixel: an empty line and a line of text round differently. */
  function same(actual: number | undefined, expected: number | undefined) {
    expect(actual).toBeDefined()
    expect(Math.abs((actual ?? 0) - (expected ?? 0))).toBeLessThan(1)
  }

  /** The vertical position of each part, in one scenario. */
  async function places(page: Page, scenario: Scenario, title: string, cover: string) {
    await open(page, scenario)
    const y = async (selector: Parameters<Page['getByRole']>) => (await box(page, selector)).y
    const upNext = page.getByRole('heading', { name: 'Up next' })
    return {
      cover: await y(['img', { name: cover }]),
      title: await y(['heading', { name: title }]),
      artists: (await page.getByText('Example Artist', { exact: true }).first().boundingBox())?.y,
      progress: await y(['progressbar']),
      upNext: (await upNext.count()) > 0 ? (await upNext.boundingBox())?.y : undefined,
    }
  }

  test('no queue', async ({ page }) => {
    const queue = await places(page, 'playing-with-queue', 'Example Song', 'Example Album')
    const none = await places(page, 'cast-without-queue', 'Cast Song', 'Example Album')
    same(none.cover, queue.cover)
    same(none.title, queue.title)
    same(none.progress, queue.progress)
  })

  test('long title', async ({ page }) => {
    const short = await places(page, 'playing-with-queue', 'Example Song', 'Example Album')
    const long = await places(
      page,
      'long-title',
      'A Remarkably Long Song Title That Keeps Going Well Past Two Lines On Any Reasonable Screen (Extended Live Version)',
      'An Album Title That Is Also Far Too Long To Fit On One Line',
    )
    same(long.cover, short.cover)
    same(long.progress, short.progress)
    same(long.upNext, short.upNext)
  })

  test('missing album', async ({ page }) => {
    const album = await places(page, 'playing-with-queue', 'Example Song', 'Example Album')
    const none = await places(page, 'no-album', 'Single Song', 'Example Artist')
    same(none.cover, album.cover)
    same(none.progress, album.progress)
    same(none.upNext, album.upNext)
    await screenshot(page, 'no-album')
  })
})
