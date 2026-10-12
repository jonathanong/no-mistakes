import { fixtureTemplate } from "./fixture-source.mjs";
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
    if (process.platform !== 'linux') throw new Error('SDK process ownership qualification requires Linux');
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
            await writeFile(join(helper, 'package.json'), fixtureTemplate("package-manifest.txt", [JSON.stringify('browser-registrar')]));
            await writeFile(join(helper, 'index.mjs'), registrar);
        }
        const nearPrefix = join(directory, 'node_modules/vitest-extra');
        await mkdir(nearPrefix);
        await writeFile(join(nearPrefix, 'package.json'), fixtureTemplate("package-manifest.txt", [JSON.stringify('vitest-extra')]));
        await writeFile(join(nearPrefix, 'index.mjs'), fixtureTemplate("browser-timeout-child-template-3.txt"));
        const story = storybook ? await prepareStorybookFixture(directory, storybook) : undefined;
        const plugin = fileURLToPath(new URL('../../packages/no-mistakes/test-timeout-policy/plugin.mjs', import.meta.url));
        await writeFile(join(directory, 'vitest.config.mts'), fixtureTemplate("browser-timeout-child-template-2.txt", [JSON.stringify(plugin), story?.imports ?? '', JSON.stringify([...(registrar ? ['browser-registrar'] : []), ...(story?.registrations ?? [])]), JSON.stringify([directory, fileURLToPath(new URL('../../../', import.meta.url))]), JSON.stringify(['vitest-extra', ...(story ? ['react', 'react-dom', ...story.optimizeIncludes] : [])]), JSON.stringify(story?.alias ?? {}), story?.plugins ?? '', configTimeout]));
        if (!storybook)
            await writeFile(join(directory, 'case.test.mts'), fixtureTemplate("browser-timeout-child-template-1.txt", [source]));
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
