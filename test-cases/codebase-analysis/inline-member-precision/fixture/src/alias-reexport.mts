import { primary } from './alias-provider.mjs';
function replacement() {}
primary.set = replacement;
function f(client = primary) { const alias = client; alias.get(); }
f();
