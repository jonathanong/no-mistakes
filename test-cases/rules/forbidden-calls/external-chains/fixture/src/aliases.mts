import * as ns from 'some-pkg'
import value, { named } from 'some-pkg'

const nsFull = ns.a.b
const nsPartial = ns.a
const defaultFull = value.a.b
const defaultPartial = value.a
const nsDefault = ns.default
const defaultSingle = value.b
const namedMethod = named.method

export function aliases() {
  nsFull()
  nsPartial.b()
  defaultFull()
  defaultPartial.b()
  nsDefault.b()
  defaultSingle()
  namedMethod()
}
