import { make } from './uncertain-factory-provider.mts';
import { unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const api = { create: make };
const other = { create: replacement };
const clean = { create: make };
function mutate(value) { value.create = replacement; }
function forward(value) { return value; }
let args = [api];
if (flag) { args = [other]; }
mutate(...args);
// The helper must agree on the same executor with and without uncertainty.
const made = forward(api.create());
forward(clean.create());
made();
unrelated.get('control');
