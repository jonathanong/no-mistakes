// These type-only, declaration, and unrecognized-call forms must each remain safe-empty.
export { type IgnoredProject } from './missing-projects'
export function ignoredProject() {}

export default unknownWorkspace()
