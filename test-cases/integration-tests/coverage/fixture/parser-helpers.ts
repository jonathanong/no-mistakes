let noInit;
const { destructured } = source
const nested = ({ name: 'nested' })
const cyclic = cyclic
const wrapped = defineConfig((nested))
exports.default = nested

export default (wrapped)

export type OnlyType = string

const object = {
  ...extraObject,
  ['computed']: 'skip',
  method() {},
  name: (`literal`),
  "quoted": 'ok',
  list: [`one`, 'two'],
  emptyList: [],
  wrappedList: ([`three`]),
  spreadList: ['one', ...extra, , `two`],
  wrappedSpreadList: (([`four`])),
  nonArray: 1,
  badList: [1],
  nested,
  cyclic,
  projects: [{ name: 'one' }, 1],
}
