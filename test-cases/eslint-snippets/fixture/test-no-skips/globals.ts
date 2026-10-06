it.skip('skip', () => {});
test.runIf(true)('condition', () => {});
describe.only('suite', () => {});
it('context', ctx => ctx.skip());
const ordinary = () => { const test = { skip() {} }; test.skip(); };
