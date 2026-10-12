import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';
/** Writes real SDK config and a final default-export runner; never constructs it in the fixture. */
export async function prepareRunnerRoute(directory, plugin, route, config) {
    const finalRunner = join(directory, 'final-runner.mts');
    const alternate = join(directory, 'alternate-runner.mts');
    await writeFile(finalRunner, `
    import { TestRunner } from 'vitest'
    import { writeFileSync } from 'node:fs'
    export default class FinalRunner extends TestRunner {
      #marker = 'own field initialized'
      config = { ...this.config, ${route.configTimeout === undefined ? '' : `testTimeout: ${route.configTimeout},`} }
      extendTaskContext = (context) => {
        ${route.taskTimeout === undefined ? '' : `context.task.timeout = ${route.taskTimeout}`}
        const extended = super.extendTaskContext(context)
        extended.finalMarker = this.#marker
        writeFileSync('extension.marker', this.#marker)
        return extended
      }
      constructor(config) {
        super(config)
        ${route.assertDirectNewTarget ? `if (new.target !== FinalRunner) throw new Error('FINAL_NEW_TARGET_CHANGED')` : ''}
        if (Object.getPrototypeOf(FinalRunner.prototype) !== TestRunner.prototype)
          throw new Error('public TestRunner base identity changed')
        writeFileSync('lifecycle.marker', 'final constructor initialized')
      }
    }
  `);
    await writeFile(alternate, `
    import { TestRunner } from 'vitest'
    export default TestRunner
  `);
    const project = `
    plugins: [testTimeoutPolicyPlugin(30000, []${route.implicitNative ? '' : `, { runner: ${route.nativeDefault ? 'null' : JSON.stringify(finalRunner)} }`})
      ${route.mergedOverride ? `, { name: 'override-runner', config: () => ({ test: { runner: ${JSON.stringify(alternate)} } }) }` : ''}],
    test: { ...${config}, ${route.nativeDefault ? '' : `runner: ${JSON.stringify(finalRunner)},`} }
  `;
    const header = `import { testTimeoutPolicyPlugin } from ${JSON.stringify(plugin)}
`;
    if (route.standalone) {
        await writeFile(join(directory, 'vitest.standalone.config.mts'), header + `export default { ${project} }`);
        await writeFile(join(directory, 'vitest.config.mts'), `export default { test: { projects: ['./vitest.standalone.config.mts'] } }`);
    }
    else {
        await writeFile(join(directory, 'vitest.config.mts'), header + `export default { ${project} }`);
    }
    return route.cliOverride ? ['--runner', alternate] : [];
}
