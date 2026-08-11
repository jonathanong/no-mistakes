// no-mistakes-disable-file integration-test-no-mocks: policy fixture
import { test } from 'vitest'
import { callOpenAI } from '../helpers/openai.mts'

test('uses a disallowed integration', async () => {
  await callOpenAI()
})
