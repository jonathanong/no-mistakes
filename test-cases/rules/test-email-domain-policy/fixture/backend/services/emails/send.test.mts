import { it } from 'vitest'

it('uses recipients', () => [
  'person@example.com',
  'person%40example%2Ecom',
  'tests+person@voucha.ai',
  'tests%2Bperson%40voucha.ai',
  'https://example.com/path',
  'user@example.company',
])
