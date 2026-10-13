import { read } from '@example/db'
import { buildImportedRead } from './imported-builder'

export function extensionlessRead() {
  return read(buildImportedRead())
}
