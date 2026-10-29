import './automock-initializers-dependency.mts'
import { forbidden } from './forbidden.mts'

forbidden()

export function analyze() {
  return 'mocked'
}
