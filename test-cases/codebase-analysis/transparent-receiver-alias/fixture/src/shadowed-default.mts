import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
function outer() { const primary = { get: replacement }; function f(client = primary) { const alias = client; alias.get(); } f(); }
outer();
