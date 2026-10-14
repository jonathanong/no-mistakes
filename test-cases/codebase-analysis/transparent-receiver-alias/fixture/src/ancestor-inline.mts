import { primary } from '@vendor/valkey';
function replacement() {}
primary.nested = {get: replacement};
((client = primary) => { const alias = client; alias.nested.get(); })();
