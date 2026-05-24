import { test, vi } from 'vitest'
import { TopicEditTabs } from '../src/next-dynamic-colocated'

vi.mock('next/dynamic', () => ({
  default: () =>
    function MockDynamicComponent() {
      return null
    },
}))

vi.mock('@lib/topic-edit-tabs', () => ({
  default: () => null,
}))

test('colocated dynamic import mock covers source file', () => {
  TopicEditTabs
})
