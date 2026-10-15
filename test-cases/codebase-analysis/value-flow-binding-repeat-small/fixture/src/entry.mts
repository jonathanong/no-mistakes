import { client } from "@vendor/valkey";
function replacement() {}
function run() {
  for (const value of [0, 1]) {
    const saved = client.get;
    (function consume(fn) {
      fn("key-0");
      fn("key-1");
      fn("key-2");
      fn("key-3");
      fn("key-4");
      fn("key-5");
      fn("key-6");
      fn("key-7");
      fn("key-8");
      fn("key-9");
      fn("key-10");
      fn("key-11");
      fn("key-12");
      fn("key-13");
      fn("key-14");
      fn("key-15");
    })(saved);
    client.get = replacement;
  }
}
run();
