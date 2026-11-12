import { read } from '@bench/db'
// The finite domain is over budget, but the structural comment proof still applies.
for (const [name, body] of [
  ['n01', 'x'], ['n02', 'x'], ['n03', 'x'], ['n04', 'x'],
  ['n05', 'x'], ['n06', 'x'], ['n07', 'x'], ['n08', 'x'],
  ['n09', 'x'], ['n10', 'x'], ['n11', 'x'], ['n12', 'x'],
  ['n13', 'x'], ['n14', 'x'], ['n15', 'x'], ['n16', 'x'],
  ['n17', 'x'],
] as const) {
  read(`/* ${name} */ ${body}`) // low: true
}
