import { used } from '../target'
// Explicitly querying this file must not make it a caller in sibling reports.
used('ignored sibling')
export function own() { return 'own' }
