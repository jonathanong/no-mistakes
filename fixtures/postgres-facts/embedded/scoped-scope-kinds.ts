import { openTransaction, type TxExecutor } from '@example/db'

export class Holder {
  static {
    const tx = openTransaction()
    tx('SELECT id FROM static_block')
  }
}

export function switched(kind: string) {
  switch (kind) {
    case 'a':
      const tx = openTransaction()
      return tx('SELECT id FROM switch_case')
    default:
      return null
  }
}

export function looped() {
  for (let tx = openTransaction(), i = 0; i < 1; i += 1) {
    tx('SELECT id FROM for_init')
  }
}

// Counterintuitive on purpose: `var` hoists out of its block, so it is not a
// block-scoped binding and must not be scanned.
export function hoisted() {
  var tx = openTransaction()
  return tx('SELECT id FROM var_declaration')
}

// Only the quoted key binds. The rest must not: a destructured declarator, a
// nested pattern, a computed key, a method signature, a differently named
// property, and a tuple type.
export function untracked(
  { run: { inner } }: { run: TxExecutor },
  { [dynamicKey]: computed }: { [dynamicKey]: TxExecutor },
  { method }: { method(): TxExecutor },
  { 'quoted': quoted }: { 'quoted': TxExecutor },
  { other }: { renamed: TxExecutor },
  tuple: [TxExecutor],
) {
  const { destructured } = openTransaction()
  inner('SELECT id FROM nested_pattern')
  computed('SELECT id FROM computed_key')
  method('SELECT id FROM method_signature')
  quoted('SELECT id FROM quoted_key')
  other('SELECT id FROM other_property')
  tuple('SELECT id FROM tuple_type')
  destructured('SELECT id FROM destructured_declarator')
}

// An untyped parameter and an object pattern typed by a reference (not an
// inline literal) have no property types to read, so neither binds.
export function unreadable(untyped, { named }: Options) {
  untyped('SELECT id FROM untyped_param')
  named('SELECT id FROM referenced_pattern')
}
