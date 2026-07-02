const message = 'vi.mock should only be text'
const other = "import('msw') is documentation"
/*
vi.fn()
*/
/* setup */ vi.mock('../module')
const server = await import('msw/node')
