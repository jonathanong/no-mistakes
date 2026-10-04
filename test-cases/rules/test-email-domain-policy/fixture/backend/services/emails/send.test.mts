import { it } from 'vitest'

it('uses recipients', () => [
  'person@example.com',
  'person%40example%2Ecom',
  'tests+person@example.com',
  'tests%2Bperson%40example.com',
  'https://example.com/path',
  'user@example.company',
])
