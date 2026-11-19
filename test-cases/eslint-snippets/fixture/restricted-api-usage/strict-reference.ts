declare const api: any;
declare const other: any;
const later = api.blockingWait;
const viaWrapper = (api.blockingWait as unknown)!;
const bare = blockingWait;
const shorthand = { blockingWait };
const element = api["blockingWait"];
// Controls: calls, constructs, bindings, other names and dynamic keys stay silent.
api.wait();
api.blockingWait();
api?.blockingWait!();
new api.blockingWait();
const { blockingWait: renamed } = other;
function declared(blockingWait: number, { blockingWait: inner }: any, [blockingWait2]: any) {}
const { x = blockingWait } = other;
try {
} catch (blockingWait) {}
label: for (;;) {
  break label;
}
other.blockingWaitLater;
api[name];
type Alias = typeof api;
