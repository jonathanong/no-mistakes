import { it as check, test, describe as suite } from 'vitest';
check.skip('skip', () => {});
check.skipIf(process.env.CI)('conditional', () => {});
suite.runIf(process.env.KEY)('suite', () => {});
test.only('focused', () => {});
test.todo('pending');
test.fixme('fixme', () => {});
check.each([1]).skip('table', () => {});
check.skip.each([1])('table', () => {});
check.each`value\n${1}`.only('table', () => {});
suite.skipIf(true).each([1])('suite', () => {});
check('ctx', context => { if (!process.env.KEY) context.skip(); });
check.each([1])('ctx', (_, ctx) => { ctx.skip(); }); // Table data is not the context.
check('shadow', ctx => { function helper(ctx) { ctx.skip(); } });
check('capture', ctx => { const nested = () => ctx.skip(); nested(); });
