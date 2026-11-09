import { sink as importedSink, Timer, make, relay } from './sink.mjs';
export function run() {
  importedSink(); importedSink();
  relay(importedSink);
  new Timer().fire();
  make()();
  const name = Math.random() ? 'a' : 'b';
  unknownReceiver[name]();
}
run();
