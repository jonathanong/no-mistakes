import { primary } from '@vendor/valkey';
function replacement() {}
// The matching write happens before the method is copied.
((client = primary) => { primary.get = replacement; const get = client.get; get(); })();
