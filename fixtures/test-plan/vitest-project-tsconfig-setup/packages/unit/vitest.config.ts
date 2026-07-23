import defaultValues from '@setup/default-values'
import * as namespaceValues from '@setup/namespace-values'

export default {
  test: {
    name: 'unit',
    include: ['tests/**/*.test.ts'],
    setupFiles: [defaultValues.files, namespaceValues.files],
  },
}
