import { type ChildProcess, spawn } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { SCENARIOS, WITH_LOGOS, portFor } from './scenarios.ts'

const BINARY =
  process.env.JUKELIGHT_BIN ??
  fileURLToPath(new URL('../../../target/debug/jukelight', import.meta.url))

async function waitForBackend(port: number): Promise<void> {
  const deadline = Date.now() + 15_000
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/api/v1/player`)
      if (response.ok) return
    } catch {
      // Not listening yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 100))
  }
  throw new Error(`backend on port ${port} did not start`)
}

/**
 * A folder of test logos: plain shapes in the brands' places, so the tests
 * see logos without the repository holding any real one.
 */
function testLogos(): string {
  const folder = mkdtempSync(join(tmpdir(), 'jukelight-test-logos-'))
  // An empty folder for the scenarios without logos.
  mkdirSync(join(folder, 'none'))
  const shape = (width: number, height: number, fill: string) =>
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}"><rect width="${width}" height="${height}" rx="${height / 2}" fill="${fill}"/></svg>`
  writeFileSync(join(folder, 'qobuz.svg'), shape(100, 40, '#ffffff'))
  writeFileSync(join(folder, 'music-assistant.svg'), shape(40, 40, '#18bcf2'))
  writeFileSync(join(folder, 'hi-res.svg'), shape(40, 40, '#d4a537'))
  writeFileSync(join(folder, 'bluesound.svg'), shape(200, 20, '#ffffff'))
  return folder
}

/** Starts one backend per scenario; returns the teardown. */
export default async function globalSetup(): Promise<() => void> {
  if (!existsSync(BINARY)) {
    throw new Error(`${BINARY} not found: run \`cargo build\` first (or set JUKELIGHT_BIN)`)
  }
  const logos = testLogos()
  const processes: ChildProcess[] = SCENARIOS.map((scenario) =>
    spawn(BINARY, [], {
      env: {
        ...process.env,
        JUKELIGHT_DEVICE: `fake:${scenario}`,
        HTTP_PORT: String(portFor(scenario)),
        LOG_LEVEL: 'warn',
        BRAND_ASSETS_DIR: WITH_LOGOS.includes(scenario) ? logos : join(logos, 'none'),
      },
      stdio: 'inherit',
    }),
  )
  const stopAll = () => {
    for (const child of processes) child.kill()
    rmSync(logos, { recursive: true, force: true })
  }
  try {
    await Promise.all(SCENARIOS.map((scenario) => waitForBackend(portFor(scenario))))
  } catch (error) {
    // Without this, backends that did start would keep holding their ports.
    stopAll()
    throw error
  }
  return stopAll
}
