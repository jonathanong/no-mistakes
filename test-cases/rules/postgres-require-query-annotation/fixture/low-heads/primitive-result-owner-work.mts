import { read } from '@bench/db'

function impossible(value) { throw new Error(String(value)) }
function named(type) {
  switch (type) {
    case 'one': return `/* readOne */ SELECT 1`
    case 'two': return `/* readTwo */ SELECT 2`
    default: return impossible(type)
  }
}

// The root export has the same spelling as each local result; scope identity wins.
export const q = 'unrelated'
export const unrelatedA = 'a'
export const unrelatedB = 'b'

function first(type) {
  const q = named(type)
  read(q) // low: true
}
function second(type) {
  const q = named(type)
  read(q) // low: true
}
function third(type) {
  const q = named(type)
  read(q) // low: true
}
function fourth(type) {
  const q = named(type)
  read(q) // low: true
}
function fifth(type) {
  const q = named(type)
  read(q) // low: true
}
function sixth(type) {
  const q = named(type)
  read(q) // low: true
}

function rejected(type) {
  const q = named(type)
  read(q) // low: false
  read(q) // low: false
  read(q) // low: false
  read(q) // low: false
  read(q) // low: false
  read(q) // low: false
}
