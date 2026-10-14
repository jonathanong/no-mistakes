import { primary } from '@vendor/valkey';
function replacement() {}
// A loop test can execute the inline acquisition again before the next body.
for (let index = 0; (index < 2 && ((client = primary) => { const get = client.get; primary.get = replacement; get(); return true; })()); index++) {}
