import { make } from './uncertain-factory-provider.mts';
import { primary, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const api = { create: make };
const other = { create: replacement };
function mutate(value) { value.create = replacement; }
let args = [api];
if (flag) { args = [other]; }
mutate(...args);
// Immediate factory admission must see through the uncertain callable wrapper.
api.create()();
unrelated.get('control');

function execute(value) { value(); }
function build() { return execute; }
const local = { create: build };
let localArgs = [local];
if (flag) { localArgs = [other]; }
mutate(...localArgs);
// Its returned repository callable requires admission of the uncertain factory.
local.create()(primary.get);
