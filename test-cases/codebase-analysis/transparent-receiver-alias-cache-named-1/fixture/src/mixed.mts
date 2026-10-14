import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
function f(client = primary) {
const first = client; const alias = first;
alias.get(); alias.set();
}
f();
