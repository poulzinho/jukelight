import { type ChildProcess, spawn } from 'node:child_process'
import { existsSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { SCENARIOS, portFor } from './scenarios.ts'

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

/** Starts one backend per scenario; returns the teardown. */
export default async function globalSetup(): Promise<() => void> {
  if (!existsSync(BINARY)) {
    throw new Error(`${BINARY} not found: run \`cargo build\` first (or set JUKELIGHT_BIN)`)
  }
  const processes: ChildProcess[] = SCENARIOS.map((scenario) =>
    spawn(BINARY, [], {
      env: {
        ...process.env,
        JUKELIGHT_DEVICE: `fake:${scenario}`,
        HTTP_PORT: String(portFor(scenario)),
        LOG_LEVEL: 'warn',
      },
      stdio: 'inherit',
    }),
  )
  const stopAll = () => {
    for (const child of processes) child.kill()
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
