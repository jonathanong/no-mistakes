import './resolved-helper.ts'
// This runtime import cycle must remain bounded during setup closure discovery.
import './resolved-cycle.ts'

export const resolvedSetup = true
