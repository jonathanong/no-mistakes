import { it } from 'vitest'
import { loadPage } from './client'
it('outside runner ownership', async () => { await loadPage('', '/unregistered') })
