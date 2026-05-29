import { defineConfig } from 'vitest/config'
// @ts-ignore
import projects from './vitest.cjs-non-module-exports-source.cjs'

// The CJS helper uses exports.x = ... (not module.exports = ...)
// When looking for its "default" export, the ExpressionStatement is an assignment
// but NOT module.exports, so commonjs_default_expression returns None
// covering lines 72 (closing brace of if-let-Some) and 118 (return None)
export default defineConfig({
  test: {
    projects,
  },
})
