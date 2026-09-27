import { loadPage } from './client'
declare const it: (name: string, callback: () => Promise<void>) => void
it('not the runner', async () => { await loadPage('', '/unbound') })
