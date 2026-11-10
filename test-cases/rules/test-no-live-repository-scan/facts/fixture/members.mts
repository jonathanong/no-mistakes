import fs, { mkdtempSync, mkdtempDisposableSync, readdirSync } from 'node:fs'
function workspace() { return { root: mkdtempSync('case-') } }
async function asyncWorkspace() { return { root: mkdtempSync('case-') } }
const { root: awaited } = await asyncWorkspace()
readdirSync(awaited)
readdirSync(fs.mkdtempDisposableSync('case-').path)
using disposable = mkdtempDisposableSync('case-')
readdirSync(disposable.path)
export const exposed = { root: mkdtempSync('case-') }
readdirSync(exposed.root)
const spread = { ...workspace() }
readdirSync(spread.root)
const computed = { ['root']: mkdtempSync('case-') }
readdirSync(computed.root)
const method = { root() { return mkdtempSync('case-') } }
readdirSync(method.root)
const accessor = { get root() { return mkdtempSync('case-') } }
readdirSync(accessor.root)
const { root: restRoot, ...rest } = workspace()
readdirSync(restRoot)
const { root: defaultRoot = '/repository' } = workspace()
readdirSync(defaultRoot)
const { ['root']: computedRoot } = workspace()
readdirSync(computedRoot)
const [arrayRoot] = [mkdtempSync('case-')]
readdirSync(arrayRoot)
let rewritten = { root: mkdtempSync('case-') }
rewritten = { root: '/repository' }
readdirSync(rewritten.root)
const escaped = { root: mkdtempSync('case-') }
mutate(escaped)
readdirSync(escaped.root)
let mutable = { root: mkdtempSync('case-') }
readdirSync(mutable.root)
function missing() { return {} }
readdirSync(missing().root)
const alias0 = workspace()
const alias1 = alias0
const alias2 = alias1
const alias3 = alias2
const alias4 = alias3
const alias5 = alias4
const alias6 = alias5
const alias7 = alias6
const alias8 = alias7
const alias9 = alias8
const alias10 = alias9
const alias11 = alias10
const alias12 = alias11
const alias13 = alias12
const alias14 = alias13
const alias15 = alias14
const alias16 = alias15
const alias17 = alias16
const alias18 = alias17
const alias19 = alias18
readdirSync(alias19.root)
function getter0() { return { root: mkdtempSync('case-') } }
function getter1() { return getter0() }
function getter2() { return getter1() }
function getter3() { return getter2() }
function getter4() { return getter3() }
function getter5() { return getter4() }
function getter6() { return getter5() }
function getter7() { return getter6() }
function getter8() { return getter7() }
function getter9() { return getter8() }
function getter10() { return getter9() }
function getter11() { return getter10() }
function getter12() { return getter11() }
function getter13() { return getter12() }
function getter14() { return getter13() }
function getter15() { return getter14() }
function getter16() { return getter15() }
function getter17() { return getter16() }
function getter18() { return getter17() }
function getter19() { return getter18() }
readdirSync(getter19().root)
const { absent = mkdtempSync('case-') } = {}
readdirSync(absent)
const { omitted = '/repository' } = {}
readdirSync(omitted)
function empty() { return {} }
const { fallback = mkdtempSync('case-') } = empty()
readdirSync(fallback)
function opaque() { return { root: notAPath } }
const notAPath = {}
const { root: shadowed = mkdtempSync('case-') } = opaque()
readdirSync(shadowed)
let unknownObject = { root: mkdtempSync('case-') }
const { root: fromUnknown = mkdtempSync('case-') } = unknownObject
readdirSync(fromUnknown)
