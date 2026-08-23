// no-mistakes-disable-file test-no-unmocked-dynamic-imports: malformed legacy test
import { test } from 'vitest'

test('malformed', () => {
  const broken = {
})
