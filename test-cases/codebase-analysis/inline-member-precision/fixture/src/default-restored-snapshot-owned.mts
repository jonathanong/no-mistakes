import { primary } from '@vendor/valkey';
function replacement() {}
// Leaving the synthetic default scope must restore module evaluation for siblings.
export default {};
export const value = ((client = primary) => { const get = client.get; primary.get = replacement; get(); })();
