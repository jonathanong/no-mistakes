import { classRepeated, classOnce, classStaticRepeated } from '@vendor/valkey';
function replacement() {}
function acquireRepeated(fn = classRepeated.get) { return fn; }
function forwardRepeated() { return acquireRepeated(); }
const key = 'run';
class RepeatedRunner {
  [key]() { ((fn) => fn('key'))(forwardRepeated()); }
}
const repeatedInstance = new RepeatedRunner();
for (const value of [0, 1]) {
  repeatedInstance[key]();
  classRepeated.get = replacement;
}
function acquireOnce(fn = classOnce.get) { return fn; }
function forwardOnce() { return acquireOnce(); }
class OnceRunner {
  [key]() { ((fn) => fn('key'))(forwardOnce()); }
}
const onceInstance = new OnceRunner();
onceInstance[key]();
classOnce.get = replacement;
function acquireStatic(fn = classStaticRepeated.get) { return fn; }
function forwardStatic() { return acquireStatic(); }
class StaticRunner {
  run() { ((fn) => fn('key'))(forwardStatic()); }
}
const staticInstance = new StaticRunner();
for (const value of [0, 1]) {
  staticInstance.run();
  classStaticRepeated.get = replacement;
}
