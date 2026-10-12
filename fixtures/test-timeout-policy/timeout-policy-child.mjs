import { fixtureTemplate } from "./fixture-source.mjs";
import { prepareRunnerRoute } from './runner-route-fixture.mjs';
import { spawn } from 'node:child_process';
import { appendFile, mkdir, mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { observeSdkChild } from './owned-sdk-child.mjs';
const require = createRequire(import.meta.url);
const cli = join(dirname(require.resolve('vitest/package.json')), 'vitest.mjs');
const vitest = fileURLToPath(import.meta.resolve('vitest'));
const policy = fileURLToPath(new URL('../../packages/no-mistakes/test-timeout-policy/index.mjs', import.meta.url));
const plugin = fileURLToPath(new URL('../../packages/no-mistakes/test-timeout-policy/plugin.mjs', import.meta.url));
export async function fixture(source, config = fixtureTemplate("timeout-policy-child-template-4.txt"), extension = '', helper, observation = {}, route) {
    if (process.platform !== 'linux')
        throw new Error('SDK process ownership qualification requires Linux');
    const directory = await mkdtemp(join(tmpdir(), 'test-timeout-policy-'));
    const maximum = observation.policyMaximum ?? 30000;
    let removable = true;
    const run = async () => {
        await mkdir(join(directory, 'node_modules'));
        await symlink(dirname(require.resolve('vitest/package.json')), join(directory, 'node_modules/vitest'), 'dir');
        if (helper) {
            await mkdir(join(directory, 'workspace-helper'));
            await writeFile(join(directory, 'workspace-helper/package.json'), fixtureTemplate("package-manifest.txt", [JSON.stringify('deadline-helper')]));
            await writeFile(join(directory, 'workspace-helper/index.mjs'), helper.source);
            await symlink(join(directory, 'workspace-helper'), join(directory, 'node_modules/deadline-helper'), 'dir');
            if (helper.transitiveSource) {
                const registrar = join(directory, 'workspace-registrar');
                await mkdir(registrar);
                await writeFile(join(registrar, 'package.json'), fixtureTemplate("package-manifest.txt", [JSON.stringify('deadline-registrar')]));
                await writeFile(join(registrar, 'index.mjs'), helper.transitiveSource);
                await symlink(registrar, join(directory, 'node_modules/deadline-registrar'), 'dir');
            }
        }
        await writeFile(join(directory, 'vitest.config.mts'), fixtureTemplate("timeout-policy-child-template-3.txt", [JSON.stringify(plugin), config, maximum, JSON.stringify([
            'deadline-helper',
            ...(helper?.inlineTransitive ? ['deadline-registrar'] : []),
        ]), JSON.stringify(join(directory, 'runner.mts')), helper?.preserveSymlinks ?? false, JSON.stringify(join(directory, 'node_modules/deadline-helper/index.mjs')), JSON.stringify(join(directory, 'runner.mts'))]));
        await writeFile(join(directory, 'runner.mts'), fixtureTemplate("timeout-policy-child-template-2.txt", [JSON.stringify(vitest), JSON.stringify(policy), maximum, extension, maximum]));
        await writeFile(join(directory, 'case.test.mts'), fixtureTemplate("timeout-policy-child-template-1.txt", [JSON.stringify(vitest), source]));
        const routeArgs = route ? await prepareRunnerRoute(directory, plugin, route, config) : [];
        const child = spawn(process.execPath, [cli, 'run', '--root', directory, '--maxWorkers=1', ...routeArgs], {
            cwd: directory,
            stdio: ['ignore', 'pipe', 'pipe'],
            detached: true,
        });
        // The directory belongs to the started child until close AND group drain complete.
        removable = false;
        const outcome = await observeSdkChild(child, observation);
        removable = outcome.drained;
        if (process.env.SDK_CHILD_RECEIPT_FILE)
            await appendFile(process.env.SDK_CHILD_RECEIPT_FILE, JSON.stringify({
                pid: child.pid,
                closed: outcome.closed,
                drained: outcome.drained,
                code: outcome.result?.code,
                signal: outcome.result?.signal,
                forcedCleanup: outcome.forcedCleanup,
                errors: outcome.errors.map((error) => error instanceof Error
                    ? { name: error.name, message: error.message }
                    : { message: String(error) }),
            }) + '\n');
        observation.onFinalized?.(outcome);
        if (outcome.errors.length === 1)
            throw outcome.errors[0];
        if (outcome.errors.length > 1)
            throw new AggregateError(outcome.errors, 'SDK child failure and cleanup diagnostics', {
                cause: outcome.errors[0],
            });
        const { result, output, forcedCleanup } = outcome;
        if (!result)
            throw new Error('SDK child did not close');
        const marker = await readFile(join(directory, 'continued.marker'), 'utf8').catch(() => undefined);
        const extensionMarker = await readFile(join(directory, 'extension.marker'), 'utf8').catch(() => undefined);
        const lifecycle = await readFile(join(directory, 'lifecycle.marker'), 'utf8').catch(() => undefined);
        return { ...result, output, marker, extensionMarker, lifecycle, forcedCleanup };
    };
    const settled = await run().then((value) => ({ value }), (error) => ({ error }));
    if (removable) {
        try {
            await rm(directory, { recursive: true, force: true });
        }
        catch (error) {
            if ('error' in settled)
                throw new AggregateError([settled.error, error], 'SDK failure and resource removal failure', { cause: settled.error });
            throw error;
        }
    }
    if ('error' in settled)
        throw settled.error;
    return settled.value;
}
