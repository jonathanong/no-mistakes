import sql from 'sql-template-strings'
import otherSql from '@other/sql'
import { read } from '@bench/db'
declare const opaque: unknown, flag: boolean, rest: unknown[]

// One executor uses the three exact private literal call sites.
async function recipients(label: string, kind, ids, eligibility: ReturnType<typeof sql>) {
  const query = sql`/* `.append(label).append(sql` */ SELECT ${kind}`)
  query.append(eligibility)
  query.append(sql` ORDER BY id LIMIT ${ids?.length ?? 20}`)
  read(query) // proof: Literal
}
recipients('getTopicRecipients', 'topics', opaque, opaque)
recipients('getReferralRecipients', 'referrals', opaque, opaque)
recipients('getNewsRecipients', 'news', opaque, opaque)

async function initializerSuffix(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`).append(opaque)
  read(query) // proof: Literal
}
initializerSuffix('initializerSuffix', 1, 2, 3)

// Existing empty-root ordinary-template grammar remains independently supported.
async function legacyRun() {
  const select = async (name: string) => legacy(name)
  await Promise.all([select('first'), select('second')])
}
async function legacy(name: string) {
  const query = sql``.append(`/* legacy:${name} */ SELECT 1`)
  query.append(opaque)
  read(query) // proof: Literal
}

async function emptyName(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
emptyName('', 1, 2, 3)

async function blankName(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
blankName('  ', 1, 2, 3)

async function injectedClose(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
injectedClose('safe */ SELECT 2 /*', 1, 2, 3)

async function injectedOpen(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
injectedOpen('safe /* nested', 1, 2, 3)

async function mixedCaller(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
mixedCaller('known', 1, 2, 3)
mixedCaller(opaque, 1, 2, 3)

async function spreadCaller(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
spreadCaller('known', ...rest)

async function optionalCaller(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
optionalCaller?.('known', 1, 2, 3)

async function aliasedCaller(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
const alias = aliasedCaller; alias('known', 1, 2, 3)

export async function exportedHelper(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
exportedHelper('known', 1, 2, 3)

async function reassignedLabel(label, a, b, c) {
  label = 'other'
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
reassignedLabel('known', 1, 2, 3)

async function capturedLabel(label, a, b, c) {
  schedule(() => consume(label))
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
capturedLabel('known', 1, 2, 3)

async function aliasedQuery(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  const alias = query; consume(alias)
  read(query) // proof: Unproven
}
aliasedQuery('known', 1, 2, 3)

async function capturedQuery(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  schedule(() => query.append('x'))
  read(query) // proof: Unproven
}
capturedQuery('known', 1, 2, 3)

async function mutatedQuery(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  query.text = 'SELECT 2'
  read(query) // proof: Unproven
}
mutatedQuery('known', 1, 2, 3)

async function laterEscape(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
  consume(query)
}
laterEscape('known', 1, 2, 3)

async function reassignedHelper(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
reassignedHelper = opaque
reassignedHelper('known', 1, 2, 3)

async function wrongArity(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
wrongArity('known', 1, 2)

async function defaultParameter(label, a = 1, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
defaultParameter('known', 1, 2, 3)

async function restParameter(label, ...rest) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
restParameter('known', 1, 2, 3)

async function customRoot(label, a, b, c) {
  const query = otherSql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
customRoot('known', 1, 2, 3)

async function customClose(label, a, b, c) {
  const query = sql`/* `.append(label).append(otherSql` */ SELECT 1`)
  read(query) // proof: Unproven
}
customClose('known', 1, 2, 3)

async function bindHoleBeforeClose(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` ${opaque} */ SELECT 1`)
  read(query) // proof: Unproven
}
bindHoleBeforeClose('known', 1, 2, 3)

async function taggedNameHole(label, a, b, c) {
  const query = sql`/* ${label} */ SELECT 1`
  read(query) // proof: Unproven
}
taggedNameHole('known', 1, 2, 3)

async function unknownBeforeClose(label, a, b, c) {
  const query = sql`/* `.append(opaque).append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
unknownBeforeClose('known', 1, 2, 3)

async function tooManyCallers(label, a, b, c) {
  const query = sql`/* `.append(label).append(sql` */ SELECT 1`)
  read(query) // proof: Unproven
}
tooManyCallers('a', 1, 2, 3); tooManyCallers('b', 1, 2, 3); tooManyCallers('c', 1, 2, 3)
tooManyCallers('d', 1, 2, 3); tooManyCallers('e', 1, 2, 3); tooManyCallers('f', 1, 2, 3)
tooManyCallers('g', 1, 2, 3); tooManyCallers('h', 1, 2, 3); tooManyCallers('i', 1, 2, 3)
tooManyCallers('j', 1, 2, 3); tooManyCallers('k', 1, 2, 3); tooManyCallers('l', 1, 2, 3)
tooManyCallers('m', 1, 2, 3); tooManyCallers('n', 1, 2, 3); tooManyCallers('o', 1, 2, 3)
tooManyCallers('p', 1, 2, 3); tooManyCallers('q', 1, 2, 3)
