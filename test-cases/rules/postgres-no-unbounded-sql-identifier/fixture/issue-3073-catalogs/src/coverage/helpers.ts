const CLOSED = { github: { emailColumn: 'github_email' } } as const
function getGood(provider: keyof typeof CLOSED) { return CLOSED[provider] }
const good = getGood('github')
void good.emailColumn

const ASYNC = { github: { emailColumn: 'github_email' } } as const
async function getAsync(provider: keyof typeof ASYNC) { return ASYNC[provider] }
const asynchronous = getAsync('github')

function nonAccessor() { return process.env.COLUMN }
const UNUSED = { github: { emailColumn: 'github_email' } } as const
function getUnused(provider: keyof typeof UNUSED) { return UNUSED[provider] }

const RAW = { github: { emailColumn: 'github_email' } } as const
function getRaw(provider: keyof typeof RAW) { return RAW[provider] }
getRaw('github')

const SPREAD = { github: { emailColumn: 'github_email' } } as const
function getSpread(provider: keyof typeof SPREAD) { return SPREAD[provider] }
const keys = ['github'] as const
const spread = getSpread(...keys)
void spread.emailColumn

const WRITTEN = { github: { emailColumn: 'github_email' } } as const
function getWritten(provider: keyof typeof WRITTEN) { return WRITTEN[provider] }
const written = getWritten('github')
void written.emailColumn
getWritten = nonAccessor as any

const NAMESPACE = { github: { emailColumn: 'github_email' } } as const
export function getNamespace(provider: keyof typeof NAMESPACE) { return NAMESPACE[provider] }

const ALTERNATIVE = { github: { emailColumn: 'github_email' } } as const
export function getAlternative(provider: keyof typeof ALTERNATIVE) { return ALTERNATIVE[provider] }
declare const maybeGet: ((provider: string) => unknown) | undefined
const mixed = (maybeGet ?? getAlternative)('github')
void mixed.emailColumn
const constructed = new getAlternative('github')
void constructed.emailColumn
