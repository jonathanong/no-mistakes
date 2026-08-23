// Intentionally malformed after the import: strict config evaluation reads
// this exported setup value, but recovered facts must retain its dependency.
import { subject } from './src/subject'

export const setupFiles = ['./setup.ts']

return subject
