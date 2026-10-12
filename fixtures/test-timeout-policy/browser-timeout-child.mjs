import { prepareStorybookFixture, resolveStorybookFixturePackages, } from './storybook-timeout-fixture.mjs';
import { spawn } from 'node:child_process';
import { appendFile, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { observeSdkChild } from './owned-sdk-child.mjs';
import { readBrowserOutputs } from './browser-output-receipt.mjs';
const require = createRequire(import.meta.url);
/** Actual public Browser Mode CLI; requires approved normal provider and browser setup. */
export async function browserFixture(source, configTimeout = 5000, registrar, storybook, environment = {}) {
    const directory = await mkdtemp(join(tmpdir(), 'browser-timeout-policy-'));
    let removable = true;
    const run = async () => {
        await mkdir(join(directory, 'node_modules'));
        const packages = [
            ['vitest', require.resolve('vitest/package.json')],
            ['@vitest/browser-playwright', require.resolve('@vitest/browser-playwright/package.json')],
            ...(storybook ? resolveStorybookFixturePackages() : []),
        ];
        for (const [name, manifest] of packages) {
            await mkdir(dirname(join(directory, 'node_modules', name)), { recursive: true });
            await symlink(dirname(manifest), join(directory, 'node_modules', name), 'dir');
        }
        if (registrar) {
            const helper = join(directory, 'node_modules/browser-registrar');
            await mkdir(helper);
            await writeFile(join(helper, 'package.json'), JSON.stringify({ name: 'browser-registrar', type: 'module', exports: './index.mjs' }));
            await writeFile(join(helper, 'index.mjs'), registrar);
        }
        const nearPrefix = join(directory, 'node_modules/vitest-extra');
        await mkdir(nearPrefix);
        await writeFile(join(nearPrefix, 'package.json'), JSON.stringify({ name: 'vitest-extra', type: 'module', exports: './index.mjs' }));
        await writeFile(join(nearPrefix, 'index.mjs'), 'export const retained = true');
        const story = storybook ? await prepareStorybookFixture(directory, storybook) : undefined;
        const plugin = fileURLToPath(new URL('../../packages/no-mistakes/test-timeout-policy/plugin.mjs', import.meta.url));
        await writeFile(join(directory, 'vitest.config.mts'), `
      import { browserTestTimeoutPolicyPlugin } from ${JSON.stringify(plugin)}
      ${story?.imports ?? ''}
      import { playwright } from '@vitest/browser-playwright'
      import { configDefaults } from 'vitest/config'
      import { appendFileSync } from 'node:fs'
      const policy = browserTestTimeoutPolicyPlugin(30000, ${JSON.stringify([...(registrar ? ['browser-registrar'] : []), ...(story?.registrations ?? [])])})
      const route = policy.resolveId
      const configure = policy.config
      policy.config = (...args) => {
        const configured = configure(...args)
        const lifecycle = configured.test.reporters.at(-1)
        lifecycle.onInit = (ctx) => appendFileSync('reporters.json', JSON.stringify({
          defaults: configDefaults.reporters,
          resolved: ctx.config.reporters.map(value => Array.isArray(value) ? value[0] :
            typeof value === 'string' ? value : 'lifecycle'),
        }))
        lifecycle.onUserConsoleLog = (log) => appendFileSync('browser-console.log', log.content)
        return configured
      }
      policy.resolveId = (source, importer) => {
        const id = route(source, importer)
        if (source === 'vitest') appendFileSync('routing.jsonl', JSON.stringify({ source, importer, id }) + '\\n')
        return id
      }
      let server
      export default {
        server: { fs: { allow: ${JSON.stringify([directory, fileURLToPath(new URL('../../../', import.meta.url))])} } },
        optimizeDeps: { include: ${JSON.stringify(['vitest-extra', ...(story ? ['react', 'react-dom', ...story.optimizeIncludes] : [])])} },
        resolve: { alias: ${JSON.stringify(story?.alias ?? {})} },
        plugins: [policy, ${story?.plugins ?? ''} {
          name: 'public-browser-routing-receipt', enforce: 'post',
          configureServer(value) { server = value },
          transform(code, id) {
            if (id.includes('storybook-registrar')) appendFileSync('registrar-source.jsonl', JSON.stringify({ id, code }) + '\\n')
            if (server && this.environment.name === 'client' &&
                (id.includes('case.test.mts') || id.includes('example.stories.mjs') || id.includes('tester-'))) {
              appendFileSync('optimizer.jsonl', JSON.stringify({
                phase: 'client-transform', moduleId: id,
                server: server.config.optimizeDeps,
                clientDeclared: server.config.environments?.client?.optimizeDeps,
                client: server.environments.client.config.optimizeDeps,
                activeEnvironment: this.environment.config.optimizeDeps,
                setupFiles: server.config.test?.setupFiles,
              }) + '\\n')
            }
          },
        }],
        test: { testTimeout: ${configTimeout}, hookTimeout: 5000,
          browser: { enabled: true, provider: playwright(), headless: true,
            instances: [{ browser: 'chromium' }] } },
      }
    `);
        if (!storybook)
            await writeFile(join(directory, 'case.test.mts'), `
      import { test, expect, beforeAll, vi } from 'vitest'
      ${source}
    `);
        const cli = join(dirname(require.resolve('vitest/package.json')), 'vitest.mjs');
        const child = spawn(process.execPath, [cli, 'run', '--root', directory, '--maxWorkers=1'], {
            cwd: directory,
            stdio: ['ignore', 'pipe', 'pipe'],
            detached: true,
            env: { ...process.env, ...environment },
        });
        removable = false;
        const outcome = await observeSdkChild(child);
        removable = outcome.drained;
        if (process.env.SDK_CHILD_RECEIPT_FILE)
            await appendFile(process.env.SDK_CHILD_RECEIPT_FILE, JSON.stringify({
                pid: child.pid,
                closed: outcome.closed,
                drained: outcome.drained,
                code: outcome.result?.code,
                signal: outcome.result?.signal,
                forcedCleanup: outcome.forcedCleanup,
                GOMEMLIMIT: process.env.GOMEMLIMIT,
                storybook: storybook ?? null,
                errors: outcome.errors.map((error) => error instanceof Error
                    ? { name: error.name, message: error.message }
                    : { message: String(error) }),
            }) + '\n');
        if (outcome.errors.length)
            throw new AggregateError(outcome.errors, 'Browser SDK child cleanup failure');
        if (!outcome.result)
            throw new Error('Browser SDK child did not close');
        const records = await readBrowserOutputs(directory);
        if (process.env.SDK_CHILD_RECEIPT_FILE) {
            const artifacts = process.env.SDK_CHILD_RECEIPT_FILE + '-artifacts/' + child.pid;
            await mkdir(artifacts, { recursive: true });
            await writeFile(join(artifacts, 'routing.jsonl'), records.routing);
            await writeFile(join(artifacts, 'registrar-source.jsonl'), await readFile(join(directory, 'registrar-source.jsonl'), 'utf8').catch(() => ''));
            await writeFile(join(artifacts, 'optimizer.jsonl'), records.optimizer);
            await writeFile(join(artifacts, 'browser-console.log'), records.browserConsole);
            await writeFile(join(artifacts, 'vitest.config.mts'), await readFile(join(directory, 'vitest.config.mts')));
            await writeFile(join(artifacts, 'output.log'), outcome.output);
        }
        return {
            ...outcome.result,
            output: outcome.output,
            forcedCleanup: outcome.forcedCleanup,
            ...records,
        };
    };
    const settled = await run().then((value) => ({ value }), (error) => ({ error }));
    if (removable) {
        try {
            await rm(directory, { recursive: true, force: true });
        }
        catch (error) {
            if ('error' in settled)
                throw new AggregateError([settled.error, error], 'Browser SDK failure and removal failure', {
                    cause: settled.error,
                });
            throw error;
        }
    }
    if ('error' in settled)
        throw settled.error;
    return settled.value;
}
