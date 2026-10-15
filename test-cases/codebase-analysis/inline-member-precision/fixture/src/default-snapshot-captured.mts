import { primary } from '@vendor/valkey';
function replacement() {}
// Capturing the copied method preserves its fixed value, unlike a live receiver.
export default ((client = primary) => { const get = client.get; primary.get = replacement; return () => get(); })();
