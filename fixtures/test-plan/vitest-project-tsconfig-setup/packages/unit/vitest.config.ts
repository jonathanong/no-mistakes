import defaultValues from '@setup/default-values'
import * as namespaceValues from '@setup/namespace-values'
import setupFiles from '@setup/list'

export default {
  test: {
    name: 'unit',
    include: ['tests/**/*.test.ts'],
    setupFiles: [setupFiles, defaultValues.files, namespaceValues.files],
  },
}
