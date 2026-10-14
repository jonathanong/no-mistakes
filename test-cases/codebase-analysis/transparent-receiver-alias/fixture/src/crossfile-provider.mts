import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
export function f(client = primary) { const alias = client; alias.get(); }
