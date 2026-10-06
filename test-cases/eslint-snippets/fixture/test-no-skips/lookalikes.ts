import { it as check, describe } from 'vitest';
const data = { skip: true, skipIfDeleted: true };
check('runs', () => {}, { skipIfDeleted: true });
const api = { skip() {}, only() {} };
api.skip();
api.only();
function shadow(check) { check.skip('ordinary', () => {}); }
function local(test) { test.only(); }
const context = { skip() {} };
context.skip();
describe('suite', ctx => { ctx.skip(); });
check('named data', ({ skip }) => { skip(); });
check('unknown member', () => {});
check.data.skip();
check[dynamic].only();
api.each([1]).only();
