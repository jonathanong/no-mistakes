expect(readFileSync('package.json', 'utf8')).toContain('"no-mistakes": "^0.53.2"')
expect(readRepoFile('package.json')).toContain("\"eslint\": \"~10.9.0\"")
expect(readRepoFile('package.json')).toContain('"typescript": "5.9.0"')
expect(readRepoFile('package.json')).toBe('"vitest": "4.0.0"')
expect(readFileSync('package.json', 'utf8')).toEqual('"eslint": "10.9.0"')
expect(readRepoFile('package.json')).toContain('"version": "1.0.0", "eslint": "9.0.0"')
expect(readRepoFile('package.json')).toContain('"eslint": "9.0.0", "version": "1.0.0"')
expect(readRepoFile('package.json')).toStrictEqual('"vitest": "^4.0.0"')
expect(rootPackage.devDependencies?.['@playwright/test']).toBe('1.61.1')
expect(browserCrawlPackage.dependencies?.playwright).toBe('1.61.1')
expect(packageJson.optionalDependencies?.['fsevents']).toEqual('2.3.3')
expect(packageJson.peerDependencies.react).toEqual('19.1.0')
expect(rootPackage['devDependencies']['eslint']).toBe('10.9.0')
expect(packageJson.dependencies.react).toBe('npm:preact@10.27.1')
expect(readFileSync('package.json').toString().trim()).toContain('"eslint": "^10.9.0"')
expect(readRepoFile('package.json')).toContain('"typescript": "^5.9.0"') // current manifest pin
expect(readRepoFile('package.json')).toContain('"eslint": "^9"')
expect(packageJson.devDependencies!['eslint']).toBe('10.9.0')
expect(packageJson.devDependencies.eslint!).toBe('10.9.0')
expect(packageJson.peerDependencies.react).toBe('>=18.0.0')
expect(packageJson.peerDependencies.react).toBe('>= 18.0.0')
expect(packageJson.peerDependencies.react).toBe('>= 18.0.0 < 20.0.0')
expect(packageJson.peerDependencies.react).toBe('=18.0.0')
expect(packageJson.peerDependencies.react).toBe('>=18.0.0 =19.0.0')
expect(packageJson.peerDependencies.react).toBe('>=18.0.0 <20.0.0')
expect(packageJson.peerDependencies.react).toBe('^18.0.0 || ^19.0.0')
expect(packageJson.peerDependencies.react).toBe('18.0.0 - 19.2.0')
expect(packageJson.dependencies.localPackage).toBe('workspace:1.0.0')
expect(packageJson.dependencies.localPackage).toBe('workspace:>=1.0.0 <2.0.0')
expect(packageJson.dependencies.react).toBe('npm:preact@^10.0.0 || ^11.0.0')
expect(packageJson.devDependencies.eslint).toStrictEqual('10.9.0')
expect(packageJson.devDependencies.eslint).toBe('10.9.0-beta.1+build.5')
expect(packageJson.peerDependencies.react).toBe('>=18.0.0 <20.0.0-rc.1+build.5')
expect(packageJson.peerDependencies.eslint).toBe('>=9')
expect(packageJson.peerDependencies.eslint).toBe('^9.1')
expect(packageJson.peerDependencies.eslint).toBe('>=9 <10')
expect<string>(packageJson.dependencies.foo).toBe('1.2.3')
expect(
  packageJson.dependencies.foo, // current dependency
).toBe('1.2.3')
expect(
  packageJson.dependencies.foo, /* current dependency */
).toBe('1.2.3')
expect.poll(
  () => packageJson.dependencies.foo,
  { timeout: 25_000 },
).toBe('1.2.3')
expect(packageJson.devDependencies[dependencyName]).toBe('10.9.0')
expect(packageJson.dependencies?.[dependency.name]).toBe('1.2.3')
expect(packageJson.dependencies.foo as string).toBe('1.2.3')
expect((packageJson.dependencies.foo)).toEqual('1.2.3')
expect(((packageJson.dependencies.foo as unknown as string))).toStrictEqual('1.2.3')
expect(packageJson.devDependencies).toHaveProperty('eslint', '10.9.0')
expect((packageJson.devDependencies as DependencyMap)).toHaveProperty('eslint', '10.9.0')
expect(packageJson.devDependencies).toHaveProperty('eslint', "10.9.0")
expect(packageJson.devDependencies).toHaveProperty('eslint', '>= 10.9.0')
expect(packageJson.optionalDependencies).toHaveProperty(dependency.name, '2.3.3')
expect(packageJson).toHaveProperty('devDependencies.eslint', '10.9.0')
expect /* dependency check */ (packageJson.dependencies.foo).toBe('1.2.3')
`${expect(packageJson.dependencies.foo).toBe('1.2.3')}`
`${{ value: `${expect(packageJson.dependencies.foo).toBe('1.2.3')}` }.value}`
dedent`${expect(packageJson.dependencies.foo).toBe('1.2.3')}`
expect(packageJson.devDependencies.eslint, 'eslint must stay pinned').toBe('10.9.0')
expect(
  packageJson.devDependencies?.['@typescript-eslint/parser'],
).toEqual(
  '8.42.0',
)
