import { primary } from '@vendor/valkey';
import { snapshot } from './exported-copied-provider.mjs';
function replacement() {}
function f() { const alias = snapshot; primary.nested = {get: replacement}; alias.get(); }
f();
