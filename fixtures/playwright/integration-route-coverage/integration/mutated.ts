import { it } from 'vitest'
import { WebClient as StaticClient, WebClient as ComputedClient, WebClient as PrototypeClient } from './client'
const staticClient = new StaticClient()
staticClient.request = async () => {}
it('replaced static member', () => staticClient.request('/unregistered'))
const computedClient = new ComputedClient()
computedClient['request'] = async () => {}
it('replaced computed member', () => computedClient.request('/unregistered'))
PrototypeClient.prototype.request = async () => {}
const prototypeClient = new PrototypeClient()
it('replaced prototype member', () => prototypeClient.request('/unregistered'))
