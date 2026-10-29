import { manualForbidden } from '../forbidden.mts'
import '../manual-init.mts'
// The manual replacement has a different forbidden call from the real module.
export function analyze() { manualForbidden() }
