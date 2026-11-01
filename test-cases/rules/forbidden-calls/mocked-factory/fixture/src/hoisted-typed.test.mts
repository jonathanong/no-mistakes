import { vi } from 'vitest'
import { run } from './through.mts'
const noMistakes = vi.hoisted(() => ({ analyzeProject: vi.fn<(...args: unknown[]) => unknown>() }))
vi.mock<typeof import('external-analyzer')>(import('external-analyzer'), () => ({ ...noMistakes, default: noMistakes }) as unknown as typeof import('external-analyzer'))
run()
