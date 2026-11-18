declare const queue: any;
declare const dynamic: string;
declare function noop(...args: any[]): void;
queue.addAndWait;
const referenceOnly = queue.addAndWait;
queue[dynamic]();
const key = "addAndWait";
queue[key]();
let { addAndWait: mutable } = queue;
mutable = noop;
mutable();
const { addAndWait: wait } = queue;
function shadow(wait: Function) { wait(); }
{ const wait = noop; wait(); }
try { noop(); } catch (wait) { (wait as Function)(); }
function hoisted() { wait(); var wait = noop; }
const { addAndWait: fallback = noop } = queue;
fallback();
const { [dynamic]: computed } = queue;
computed();
before();
const { addAndWait: before } = queue;
const { addAndWait: overwritten } = queue;
// @ts-expect-error immutable aliases with writes are not trusted
overwritten = noop;
overwritten();
