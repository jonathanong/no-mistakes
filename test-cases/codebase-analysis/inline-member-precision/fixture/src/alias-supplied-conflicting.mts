import { primary, session } from '@vendor/valkey';
function f(client) { const alias = client; alias.get(); }
f(primary);
f(session);
