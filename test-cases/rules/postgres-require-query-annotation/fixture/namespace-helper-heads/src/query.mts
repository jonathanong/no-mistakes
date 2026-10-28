import { read } from '@bench/db'
import * as good from './barrel.mjs'
import * as wrong from './barrel-wrong.mjs'
import * as duplicate from './barrel-duplicate.mjs'
import * as typeOnly from './barrel-type-only.mjs'
import * as cyclic from './barrel-cycle-a.mjs'
import * as starred from './barrel-star.mjs'
import * as mismatched from './barrel-mismatch.mjs'
import * as mutated from './barrel.mjs'
import * as defaulted from './barrel.mjs'
import * as rested from './barrel.mjs'
import * as changed from './barrel.mjs'
import * as aliased from './barrel.mjs'
import * as shadowSource from './barrel.mjs'
import * as explicit from './barrel-explicit.mjs'
import * as computed from './barrel.mjs'
import * as reassigned from './barrel.mjs'
import * as namedWrong from './barrel-named-wrong.mjs'
import * as captured from './barrel-captured.mjs'

// Both calls must resolve through the same namespace property, local export,
// imported default binding, and source helper.
const { named, missing } = good
read(named()) // low: true
read(named()) // low: true
read(missing()) // low: false
const { explicitNamed } = explicit
read(explicitNamed()) // low: true

const { named: wrongNamed } = wrong
read(wrongNamed()) // low: false
const { named: duplicateNamed } = duplicate
read(duplicateNamed()) // low: false
const { named: typeNamed } = typeOnly
read(typeNamed()) // low: false
const { named: cycleNamed } = cyclic
read(cycleNamed()) // low: false
const { named: starNamed } = starred
read(starNamed()) // low: false
const { named: mismatchNamed } = mismatched
read(mismatchNamed()) // low: false
const { named: wrongImportNamed } = namedWrong
read(wrongImportNamed()) // low: false
const { named: capturedNamed } = captured
read(capturedNamed()) // low: true

mutated.named = opaque
const { named: mutatedNamed } = mutated
read(mutatedNamed()) // low: false
const { named: defaultNamed = opaque } = defaulted
read(defaultNamed()) // low: false
const { named: restNamed, ...remaining } = rested
read(restNamed()) // low: false
const { named: changedNamed } = changed
changedNamed = opaque
read(changedNamed()) // low: false
const alias = aliased
const { named: aliasNamed } = alias
read(aliasNamed()) // low: false
const key = 'named'
const { [key]: computedNamed } = computed
read(computedNamed()) // low: false
reassigned = opaque
const { named: reassignedNamed } = reassigned
read(reassignedNamed()) // low: false
const { named: shadowedNamed } = shadowSource
function shadow() {
  const shadowedNamed = opaque
  read(shadowedNamed()) // low: false
}

const getter = { get named() { return opaque } }
const { named: getterNamed } = getter
read(getterNamed()) // low: false
// Exporting an unrelated local must leave namespace helper ownership intact.
export const unrelated = 'fixed'
