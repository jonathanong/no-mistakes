import '@local/pkg'
import 'sharp'
import { readFileSync } from 'node:fs'

readFileSync(new URL('../resource-only.txt', import.meta.url))
