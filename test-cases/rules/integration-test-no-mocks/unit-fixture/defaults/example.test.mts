import { vi } from 'vitest'
vi.mock('../module')
const fn = vi.fn()
const server = await import('msw/node')
const nock = require('nock')
import sinon from 'sinon'
import 'msw'
