import { read } from '@bench/db'
import { enabled } from './flag.mts'

const name = enabled ? 'accounts' : 'sessions'
read(`/* ${name} */ SELECT 1`) // low: true

read(`/* ${enabled ? 'posts' : 'comments'} */ SELECT 1`) // low: true

for (const table of ['accounts', 'sessions']) {
  read(`/* ${table} */ SELECT 1`) // low: true
}

function beforeDeclaration() {
  read(`/* ${deferred} */ SELECT 1`) // low: true
}
const deferred = enabled ? 'accounts' : 'sessions'

function dynamicArm(other: string) {
  const dynamic = enabled ? 'accounts' : other
  read(`/* ${dynamic} */ SELECT 1`) // low: false
}

function dynamicList(names: string[]) {
  for (const table of names) {
    read(`/* ${table} */ SELECT 1`) // low: false
  }
}

const nested = enabled ? 'accounts' : enabled ? 'sessions' : 'posts'
read(`/* ${nested} */ SELECT 1`) // low: false

for (const table of ['*/', 'accounts']) {
  read(`/* ${table} */ SELECT 1`) // low: false
}

const fixedName = 'accounts/list'
read(`/* ${fixedName} */ SELECT ${enabled}`) // low: true

function localLiteralName() {
  const operation = 'sessions/list'
  read(`/* ${operation} */ SELECT 1`) // low: true
}

read('SELECT 1') // low: false

const emptyName = ''
read(`/* ${emptyName} */ SELECT 1`) // low: false

const terminatedName = '*/'
read(`/* ${terminatedName} */ SELECT 1`) // low: false

let mutableName = 'accounts/list'
read(`/* ${mutableName} */ SELECT 1`) // low: false

function shadowedName(fixedName: string) {
  read(`/* ${fixedName} */ SELECT 1`) // low: false
}

const aliasName = fixedName
read(`/* ${aliasName} */ SELECT 1`) // low: false

export const exportedName = 'accounts/list'
read(`/* ${exportedName} */ SELECT 1`) // low: false
