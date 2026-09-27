import { it } from 'vitest'
import { WebClient } from './client'
const client = new WebClient()
it('shadow receiver', async (client: WebClient) => { await client.request('/shadow') })
