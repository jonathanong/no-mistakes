const fromCycle = require('./cjs-project-cycle-exclusions-a.cjs')

module.exports = [
  '!../vitest.cjs-cycle-excluded-project.ts',
  ...fromCycle,
]
