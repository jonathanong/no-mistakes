import { beforeEach, describe, it as scenario } from 'vitest'
import { WebClient as Client, loadPage as load } from './client'
let client: Client
const target = { id: '42' }
beforeEach(() => { client = new Client() })
describe('static routes', () => {
  scenario('health response', async () => { await client.request('/healthz') })
  scenario('copyright HTML', async () => { await load('https://example.test', '/copyright') })
  scenario('parameter route', async () => { await client.request(`/user/${target.id}/posts`) })
  scenario('literal route wins', async () => { await client.request('/user/new/posts') })
})
