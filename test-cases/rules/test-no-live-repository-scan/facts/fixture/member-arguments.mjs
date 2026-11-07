import { mkdtempSync, readdirSync } from 'node:fs'
import { inspect } from './inspect.mjs'
const root = mkdtempSync('prefix')
inspect({ z: root, a: root })
inspect({ root, ...other })
inspect({ [key]: root })
inspect({ get root() { return root } })
inspect({ method() { return root } })
inspect({ root: root, get duplicate() { return root } })
inspect({ k0: root, k1: root, k2: root, k3: root, k4: root, k5: root, k6: root, k7: root, k8: root, k9: root, k10: root, k11: root, k12: root, k13: root, k14: root, k15: root, k16: root, k17: root, k18: root, k19: root, k20: root, k21: root, k22: root, k23: root, k24: root, k25: root, k26: root, k27: root, k28: root, k29: root, k30: root, k31: root, k32: root })
inspect(...args)
