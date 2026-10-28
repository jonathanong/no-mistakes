import { read } from '@bench/db'

// Every rejected loop still reaches a configured executor with an unknown name.
async function awaited() {
  for await (const [name, body] of [['known', 'SELECT 1']]) {
    read(`/* ${name} */ ${body}`) // low: false
  }
}
for (const [name, body, ...rest] of [['known', 'SELECT 1', 'extra']]) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of ['runtime row']) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [[1, 'SELECT 1']]) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [['known', 1]]) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [['', 'SELECT 1']]) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [['known', 'SELECT 1']]) {
  read(`/* ${name.trim()} */ ${body}`) // low: false
  read(`/* ${name} ${body} */ SELECT 1`) // low: false
}
