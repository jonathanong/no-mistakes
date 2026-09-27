import { describe, it as scenario } from 'vitest'
import { WebClient, loadPage } from './client'
const client = new WebClient()
const dynamicName = 'dynamic'
const unknownPath = '/unregistered'
for (;;) { scenario('loop', () => loadPage('', '/unregistered')); break }
for (const key in {}) { scenario('loop', () => loadPage('', '/unregistered')) }
do { scenario('loop', () => loadPage('', '/unregistered')) } while (false)
switch (false) { case true: scenario('switch', () => loadPage('', '/unregistered')) }
if (false) { scenario('if', () => loadPage('', '/unregistered')) }
scenario(dynamicName, () => loadPage('', '/unregistered'))
scenario('without callback')
scenario.todo('todo', () => loadPage('', '/unregistered'))
describe('nested', () => {
  scenario(`function callback`, function () {
    loadPage('', '/' + 'other')
    false && loadPage('', '/other')
    false ? loadPage('', '/other') : null
    client.notRequest('/unregistered')
    client.inner.request('/unregistered')
    loadPage('', unknownPath)
  })
})
