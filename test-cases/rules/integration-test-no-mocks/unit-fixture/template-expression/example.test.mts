const call = `${vi.mock('../x')}`
const server = `${await import('msw/node')}`
const nested = `${condition ? `${await import('msw/node')}` : { value: require('nock') }}`
