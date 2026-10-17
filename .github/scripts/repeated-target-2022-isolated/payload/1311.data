import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const boxes = [{ client }, { client }, { client }];
Object.defineProperty(holder, 'box', { get() { return boxes.shift(); } });
let unused;
holder.box.client = {};
// First getter read consumes another container before the actual target read.
({ first: unused = holder.box, second: unused = (holder.box.client.get = replacement) } = {});
function globalPhaseDescriptorCall(value) { value.get('key'); }
globalPhaseDescriptorCall(client);
