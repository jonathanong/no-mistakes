import { fixtureTemplate } from "./fixture-source.mjs";
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
export function resolveStorybookFixturePackages() {
    const require = createRequire(import.meta.url);
    return [
        ['@storybook/addon-vitest', require.resolve('@storybook/addon-vitest/package.json')],
        ['@storybook/react-vite', require.resolve('@storybook/react-vite/package.json')],
        ['@storybook/react', require.resolve('@storybook/react/package.json')],
        ['storybook', require.resolve('storybook/package.json')],
        ['react', require.resolve('react/package.json')],
        ['react-dom', require.resolve('react-dom/package.json')],
    ];
}
export function replaceStorybookAnnotationSetup(config, stable) {
    if (!config.test)
        throw new Error('Addon-generated annotations setup missing before stable replacement');
    const setup = config.test.setupFiles;
    const files = Array.isArray(setup) ? setup : setup ? [setup] : [];
    if (!files.some((file) => file.includes('setup-file-with-project-annotations')))
        throw new Error('Addon-generated annotations setup missing before stable replacement');
    config.test.setupFiles = files.map((file) => file.includes('setup-file-with-project-annotations') ? stable : file);
}
/** Real addon generates CSF tests and annotations; no generated SDK code is copied. */
export async function prepareStorybookFixture(directory, options) {
    const configDir = join(directory, '.storybook');
    await mkdir(configDir);
    await writeFile(join(configDir, 'main.mts'), fixtureTemplate("storybook-timeout-fixture-template-9.txt"));
    for (const [name, source] of [
        ['storybook-registrar', fixtureTemplate("storybook-timeout-fixture-template-8.txt")],
        [
            'storybook-registrar-leaf',
            fixtureTemplate("storybook-timeout-fixture-template-7.txt", [options.hookTimeout ?? 250]),
        ],
    ]) {
        const folder = join(directory, 'node_modules', name);
        await mkdir(folder);
        await writeFile(join(folder, 'package.json'), fixtureTemplate("package-manifest.txt", [JSON.stringify(name)]));
        await writeFile(join(folder, 'index.mjs'), source);
    }
    await writeFile(join(configDir, 'preview.mts'), fixtureTemplate("storybook-timeout-fixture-template-6.txt"));
    await writeFile(join(directory, 'example.stories.mjs'), fixtureTemplate("storybook-timeout-fixture-template-5.txt"));
    const stable = join(configDir, 'stable.setup.mts');
    if (options.stableSetup)
        await writeFile(stable, fixtureTemplate("storybook-timeout-fixture-template-4.txt"));
    const require = createRequire(import.meta.url);
    const storybookRequire = createRequire(require.resolve('storybook/test'));
    // Public declared CJS dependencies of storybook/test need their own optimizer entries
    // when the registering parent stays transformed through the policy facade.
    const domRequire = createRequire(storybookRequire.resolve('@testing-library/dom'));
    const optimizerAliases = Object.fromEntries([
        ...['@testing-library/dom', '@testing-library/jest-dom', '@testing-library/user-event'].map((name) => [name, storybookRequire.resolve(name)]),
        ...['aria-query', 'dom-accessibility-api'].map((name) => [name, domRequire.resolve(name)]),
    ]);
    const optimizeIncludes = Object.keys(optimizerAliases);
    return {
        optimizeIncludes,
        imports: fixtureTemplate("storybook-timeout-fixture-template-3.txt", [JSON.stringify(fileURLToPath(import.meta.url))]),
        plugins: fixtureTemplate("storybook-timeout-fixture-template-1.txt", [JSON.stringify(configDir), options.stableSetup
            ? fixtureTemplate("storybook-timeout-fixture-template-2.txt", [JSON.stringify(stable)])
            : '']),
        alias: {
            ...optimizerAliases,
            'annotation-alias': join(directory, 'node_modules/storybook-registrar/index.mjs'),
        },
        registrations: [
            '@storybook/addon-vitest',
            'storybook',
            '@storybook/react-vite',
            '@storybook/react',
            'storybook-registrar',
            'storybook-registrar-leaf',
            'annotation-alias',
        ],
    };
}
