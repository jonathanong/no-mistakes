import { createRequest } from './request.mjs'

// A configured database selector does not make HTTP query parameters SQL.
export const send = () => createRequest().get('/users').query({ name: 'alice' })
