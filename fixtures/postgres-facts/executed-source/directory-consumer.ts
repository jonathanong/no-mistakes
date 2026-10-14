import { read } from '@example/db'
import { buildImportedRead } from './module-dir'

export function directoryRead() {
  return read(buildImportedRead())
}
