import * as ns from 'some-pkg'
import value, { named } from 'some-pkg'
import * as local from './local.mts'

export function calls() {
  ns.a.b()
  value.a.b()
  ns.a()
  value.a()
  named.method()
  ns.default.b()
  value.b()
  local.a.b()
}
