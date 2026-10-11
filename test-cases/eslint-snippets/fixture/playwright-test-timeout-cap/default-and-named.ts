import test from '@playwright/test';
import * as pw from '@playwright/test';
import unrelated from './ordinary-default';
test.setTimeout(60000);
pw.default.setTimeout(0);
test.setTimeout(1); pw.default.setTimeout(30000);
const body = ({}, info) => { info.setTimeout(60000); };
const alias = body;
test('named alias', alias);
function declared({}, info) { info.setTimeout(0); }
test('declaration', declared);
const legal = ({}, info) => { info.setTimeout(1); info.setTimeout(30000); };
test('legal operations', legal);
const normal = ({}, info) => { info.setTimeout(60000); };
unrelated('not SDK', normal); unrelated.setTimeout(60000);
