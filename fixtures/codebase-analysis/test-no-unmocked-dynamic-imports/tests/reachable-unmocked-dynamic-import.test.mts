import { test } from 'vitest'
import { renderUnmockedComponent } from '../src/unmocked-next-dynamic-component.mts'

test('unmocked reachable dynamic import is reported', () => {
  renderUnmockedComponent()
})
