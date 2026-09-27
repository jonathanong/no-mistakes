import { it } from 'vitest'
import { WebClient } from './lookalike'
import { loadPage } from './client'
const client = new WebClient()
it('wrong helper', async () => { await client.request('/wrong-helper') })
it.skip('skipped', async () => { await loadPage('', '/skipped') })
it('wrong argument', async () => { await loadPage('/wrong-route', '/other') })
// A function declaration is not a test registration.
function neverRegistered() { it('unregistered', async () => { await loadPage('', '/unregistered') }) }
false && it('logical unregistered', async () => { await loadPage('', '/unregistered') })
false ? it('ternary unregistered', async () => { await loadPage('', '/unregistered') }) : null
while (false) { it('loop unregistered', async () => { await loadPage('', '/unregistered') }) }
for (const item of []) { it('empty loop', async () => { await loadPage('', '/unregistered') }) }
