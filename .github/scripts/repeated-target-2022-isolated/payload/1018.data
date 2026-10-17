import { primary } from '@vendor/valkey';
function replacement() {}
const run = ((client) => { const alias = client; return () => alias.get(); })(primary);
primary.get = replacement;
run();
