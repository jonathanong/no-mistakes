// An ESM default import is not a Vitest namespace and must stay unsupported.
import vitest from 'vitest/config'

export default vitest.defineWorkspace([{
  test: { name: 'unsupported-default-import' },
}])
