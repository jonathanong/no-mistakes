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
    await writeFile(join(configDir, 'main.mts'), `export default {
    stories: ['../example.stories.mjs'], framework: '@storybook/react-vite', addons: [],
  }`);
    for (const [name, source] of [
        ['storybook-registrar', `export * from 'storybook-registrar-leaf'`],
        [
            'storybook-registrar-leaf',
            `import { beforeAll as register } from 'vitest';
      register(() => { globalThis.annotationHookRan = true }, ${options.hookTimeout ?? 250});
      export const registered = true`,
        ],
    ]) {
        const folder = join(directory, 'node_modules', name);
        await mkdir(folder);
        await writeFile(join(folder, 'package.json'), JSON.stringify({ name, type: 'module', exports: './index.mjs' }));
        await writeFile(join(folder, 'index.mjs'), source);
    }
    await writeFile(join(configDir, 'preview.mts'), `
    import { registered } from 'annotation-alias'
    export const beforeAll = () => { globalThis.projectAnnotationsRan = registered }
  `);
    await writeFile(join(directory, 'example.stories.mjs'), `
    import React from 'react'
    import { expect } from 'vitest'
    export default { title: 'TimeoutClosure', render: () => React.createElement('button', {}, 'actual story') }
    export const RealGeneratedStory = {
      play: ({ canvasElement }) => {
        expect(globalThis.annotationHookRan).toBe(true)
        expect(globalThis.projectAnnotationsRan).toBe(true)
        expect(canvasElement.textContent).toContain('actual story')
        console.log('ACTUAL_STORY_PLAY_COMPLETED')
      },
    }
  `);
    const stable = join(configDir, 'stable.setup.mts');
    if (options.stableSetup)
        await writeFile(stable, `
    import * as previewApi from 'storybook/preview-api'
    import { getProjectAnnotations } from 'virtual:/@storybook/builder-vite/project-annotations.js'
    previewApi['set' + 'ProjectAnnotations'](getProjectAnnotations())
  `);
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
        imports: `import { storybookTest } from '@storybook/addon-vitest/vitest-plugin'
      import { replaceStorybookAnnotationSetup } from ${JSON.stringify(fileURLToPath(import.meta.url))}`,
        plugins: `storybookTest({ configDir: ${JSON.stringify(configDir)} }), ${options.stableSetup
            ? `{
      name: 'stable-annotation-setup', enforce: 'post',
      config(config) {
        replaceStorybookAnnotationSetup(config, ${JSON.stringify(stable)})
      },
    },`
            : ''}`,
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
