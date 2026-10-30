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
