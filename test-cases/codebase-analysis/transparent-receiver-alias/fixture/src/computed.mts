import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
function f(client = primary) { const alias = client; alias['get'](); }
f();
