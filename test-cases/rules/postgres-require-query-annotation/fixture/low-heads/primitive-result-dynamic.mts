import { read } from '@bench/db'

function impossible(value) { throw new Error(String(value)) }
function named(type) {
  switch (type) {
    case 'one': return `/* dynamic */ SELECT 1`
    default: return impossible(type)
  }
}
const query = named('one')
eval('opaque(query)')
read(query) // low: false
