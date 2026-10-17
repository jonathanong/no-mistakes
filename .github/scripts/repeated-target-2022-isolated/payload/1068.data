import { primary } from '@vendor/valkey';
function replacement() {}
((client = primary) => { const get = client.get; get(); })();
primary.get = replacement;
