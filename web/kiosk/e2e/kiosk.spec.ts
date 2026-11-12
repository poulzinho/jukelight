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
  await expect(page.getByRole('img', { name: 'Qobuz to Example Player' })).toBeVisible()
  await expect(page.locator('[data-part="streamer-main"]')).toHaveText('16 bit / 44.1 kHz')
  await expect(page.locator('[data-part="streamer-codec"]')).toContainText('untouched')
  // Four entries in landscape; in portrait as many as fit above the footer.
  const shown = await page.getByRole('list', { name: 'Up next' }).evaluate((list) => {
    const bounds = list.getBoundingClientRect()
    return [...list.children]
      .map((entry) => entry.getBoundingClientRect())
      .filter((entry) => entry.height > 0 && entry.left < bounds.right && entry.bottom <= bounds.bottom + 1)
      .map((entry) => entry.bottom)
  })
  if (!portrait()) expect(shown.length).toBe(4)
  else expect(shown.length).toBeGreaterThanOrEqual(3)
  // The player's signature sits below everything, overlapping nothing.
  const signature = await box(page, ['contentinfo'])
  expect(Math.max(...shown)).toBeLessThanOrEqual(signature.y)
  await screenshot(page, 'playing-with-queue')
})

test('cast session without queue', async ({ page }) => {
  await open(page, 'cast-without-queue')
  await expect(page.getByRole('heading', { name: 'Cast Song' })).toBeVisible()
  await expect(page.getByRole('img', { name: 'Spotify to Example Player' })).toBeVisible()
  await expect(page.locator('[data-part="streamer-main"]')).toHaveText('160 kbps')
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

test('hi-res track shows the HI-RES tag on the streamer', async ({ page }) => {
  await open(page, 'hi-res')
  const streamer = page.locator('[data-part="streamer"]')
  await expect(streamer.getByText('HI-RES')).toBeVisible()
  await expect(streamer.getByText('24 bit / 96 kHz')).toBeVisible()
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
  await expect(page.getByText('nothing playing')).toBeVisible()
  // The fake player is a Bluesound, without a logo: the maker's name and the player's.
  await expect(page.getByTestId('device-signature')).toContainText('Bluesound')
  await expect(page.getByTestId('device-signature')).toContainText('Example Player')
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
  // The playing screen replaces the idle one once the first snapshot is in.
  await expect(page.getByRole('heading', { name: 'Example Song' })).toBeVisible()
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

  // Spec: kiosk-display, "Progress between updates", "No transition".
  test('states fade; the progress moves without a transition', async ({ page }) => {
    expect(await motion(page)).toEqual({ animation: 'fade-in', transition: '0s' })
    // Once the page has faded in, nothing in the progress animates.
    await page.waitForTimeout(1_500)
    const running = await page
      .getByRole('progressbar')
      .evaluate((element) => element.getAnimations({ subtree: true }).length)
    expect(running).toBe(0)
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

// Spec: kiosk-display, "Signal diagram", "Diagram fits" and "Pulse along the
// diagram".
test.describe('signal diagram', () => {
  /**
   * What leaves its place in the diagram: anything past the screen, under the
   * footer or into the queue's column, and text over a logo or the streamer.
   */
  async function problems(page: Page) {
    await expect(page.getByTestId('signal-diagram')).toBeVisible()
    // Let the layout settle after measuring its box and loading its logos.
    await page.waitForTimeout(400)
    return page.evaluate(() => {
      const diagram = document.querySelector('[data-testid="signal-diagram"]') as Element
      const screen = document.documentElement.getBoundingClientRect()
      const footer = document.querySelector('[data-testid="device-signature"]')?.getBoundingClientRect()
      const queue = document.querySelector('[aria-labelledby="up-next"]')?.closest('section')?.getBoundingClientRect()
      const clearance = 0.03 * Math.min(window.innerWidth, window.innerHeight)
      const found: string[] = []
      const name = (element: Element) => (element.textContent || element.getAttribute('data-part') || element.tagName).slice(0, 30)
      for (const element of diagram.querySelectorAll('text, image, rect, path, circle')) {
        if (element.closest('[data-testid="pulse"]')) continue
        const r = element.getBoundingClientRect()
        if (r.width === 0 && r.height === 0) continue
        if (r.right > screen.right || r.left < screen.left) found.push(`past the screen: ${name(element)}`)
        // 3vmin clear of the footer, less a pixel for rounding.
        if (footer && r.bottom > footer.top - clearance + 1) found.push(`near the footer: ${name(element)}`)
        if (queue && r.right > queue.left) found.push(`into the queue: ${name(element)}`)
      }
      const logos = [...diagram.querySelectorAll('image')].map((image) => image.getBoundingClientRect())
      for (const text of diagram.querySelectorAll('text')) {
        if (text.closest('[data-part="streamer"]')) continue
        const r = text.getBoundingClientRect()
        if (logos.some((l) => r.left < l.right - 1 && r.right > l.left + 1 && r.top < l.bottom - 1 && r.bottom > l.top + 1)) {
          found.push(`over a logo: ${name(text)}`)
        }
      }
      // Inside the streamer, text stays inside its card.
      const card = diagram.querySelector('[data-part="streamer"] rect')?.getBoundingClientRect()
      for (const text of diagram.querySelectorAll('[data-part="streamer"] text')) {
        const r = text.getBoundingClientRect()
        if (card && r.right > card.right - 2) found.push(`out of the streamer: ${name(text)}`)
      }
      return found
    })
  }

  const scenarios = [
    'group',
    'group-dropped',
    'group-many',
    'group-long-names',
    'music-assistant',
    'music-assistant-unavailable',
    'upsampled',
    'long-signal',
    'unknown-quality',
    'cast-without-queue',
    'airplay',
    'bluetooth',
    'radio',
    'hi-res',
  ] as const
  for (const scenario of scenarios) {
    test(`${scenario}: everything in its place`, async ({ page }) => {
      await open(page, scenario)
      expect(await problems(page)).toEqual([])
      await screenshot(page, scenario)
    })
  }

  test('a group: every speaker, the streamer and its outputs', async ({ page }) => {
    await open(page, 'group')
    const diagram = page.getByTestId('signal-diagram')
    await expect(diagram).toHaveAttribute('data-direction', 'down')
    await expect(diagram.locator('[data-part="speaker-name"]')).toHaveText(['Kitchen', 'Office'])
    await expect(diagram.locator('[data-part="output"]')).toHaveText(['Example Speaker', 'Subwoofer'])
  })

  test('many speakers fold into "+ N more"', async ({ page }) => {
    await open(page, 'group-many')
    await expect(page.locator('[data-part="more"]')).toHaveText(/^\+ \d more$/)
  })

  test('no queue: across the width when it fits', async ({ page }) => {
    await open(page, 'airplay')
    // The landscape column is too narrow for the outputs' names beside the streamer.
    await expect(page.getByTestId('signal-diagram')).toHaveAttribute('data-direction', portrait() ? 'across' : 'down')
  })

  test('the diagram starts level with the progress bar', async ({ page }) => {
    await open(page, 'group')
    const bar = await page.getByRole('progressbar').boundingBox()
    // The drawing's left edge: a source's logo starts there (an icon has a small inset).
    const drawing = await page.getByTestId('signal-diagram').getByRole('img').boundingBox()
    expect(Math.abs((drawing?.x ?? 0) - (bar?.x ?? 0))).toBeLessThan(2)
    // Its leftmost part, here the source's icon, within that inset of it.
    const start = await page.getByTestId('signal-diagram').evaluate((diagram) =>
      Math.min(...[...diagram.querySelectorAll('svg[role="img"] *')].map((element) => element.getBoundingClientRect()).filter((r) => r.width > 0).map((r) => r.left)),
    )
    expect(start - (drawing?.x ?? 0)).toBeLessThan(20)
  })

  test('the diagram and "Up next" are top-aligned', async ({ page }) => {
    await open(page, 'group')
    // Once the layout has settled (a slow runner may read it mid-way).
    await expect
      .poll(async () => {
        const diagram = await page.getByTestId('signal-diagram').boundingBox()
        const heading = await page.getByRole('heading', { name: 'Up next' }).boundingBox()
        // Not measurable yet: never aligned.
        if (!diagram || !heading) return Number.POSITIVE_INFINITY
        return Math.abs(diagram.y - heading.y)
      }, { timeout: 5_000 })
      .toBeLessThan(4)
  })

  /** The CSS properties the diagram's running animations change. */
  async function running(page: Page) {
    return page.evaluate(() =>
      document
        .getAnimations()
        .filter((animation) => {
          if (animation.playState !== 'running') return false
          const target = (animation.effect as KeyframeEffect | null)?.target
          return target instanceof Element && target.closest('[data-testid="signal-diagram"]') !== null
        })
        .flatMap((animation) =>
          (animation.effect as KeyframeEffect)
            .getKeyframes()
            .flatMap((frame) => Object.keys(frame).filter((key) => !['offset', 'computedOffset', 'easing', 'composite'].includes(key))),
        ),
    )
  }

  test('reduced motion: nothing moves', async ({ page }) => {
    await open(page, 'group')
    await page.waitForTimeout(300)
    expect(await running(page)).toEqual([])
  })

  test.describe('without reduced motion', () => {
    test.use({ reducedMotion: 'no-preference' })

    test('the pulse runs, animating only transform and opacity, in steps', async ({ page }) => {
      await open(page, 'group')
      let properties: string[] = []
      await expect.poll(async () => (properties = await running(page)).length, { timeout: 5_000 }).toBeGreaterThan(0)
      expect([...new Set(properties)].sort()).toEqual(['opacity', 'transform'])
      const timing = await page.evaluate(() =>
        [...document.querySelectorAll('[data-part="spark"]')].map((spark) => getComputedStyle(spark).animationTimingFunction),
      )
      expect(timing.every((value) => value.startsWith('steps('))).toBe(true)
    })

    test('the pulse runs from the source to the outputs', async ({ page }) => {
      await open(page, 'group')
      await expect(page.locator('[data-part="spark"]').first()).toBeAttached()
      const delays = await page.evaluate(() =>
        [...document.querySelectorAll('[data-part="spark"]')].map((spark) => (spark as HTMLElement).style.animationDelay),
      )
      // From the source on: the outputs light up after every spark has set off.
      const arrivals = await page.evaluate(() =>
        [...document.querySelectorAll('[data-part="arrive"]')].map((port) => parseInt((port as HTMLElement).style.animationDelay, 10)),
      )
      expect(delays[0]).toBe('0ms')
      expect(Math.max(...arrivals)).toBeGreaterThan(Math.max(...delays.map((delay) => parseInt(delay, 10))))
    })

    test('between pulses: after the first pulse nothing animates while nothing changes', async ({ page }) => {
      test.setTimeout(30_000)
      await open(page, 'group')
      await expect.poll(async () => (await running(page)).length, { timeout: 5_000 }).toBeGreaterThan(0)
      await expect.poll(async () => (await running(page)).length, { timeout: 15_000 }).toBe(0)
      for (let second = 0; second < 5; second++) {
        await page.waitForTimeout(1_000)
        expect(await running(page)).toEqual([])
      }
    })
  })
})
