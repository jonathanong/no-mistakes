const call = `${vi.mock('../x')}`
const server = `${await import('msw/node')}`
const nested = `${condition ? `${await import('msw/node')}` : { value: require('nock') }}`
const commentedBlock = `${/* await import('msw/node') */ value}`
const commentedLine = `${// require('nock')
value}`
