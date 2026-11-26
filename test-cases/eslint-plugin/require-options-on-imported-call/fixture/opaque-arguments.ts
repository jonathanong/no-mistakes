import { enqueue } from "@fixture/jobs";
declare const prefix: [string] | [string, { timeoutMs?: number }];

enqueue("syntactically-known", { timeoutMs: 200 });
enqueue("sample", ...prefix);
enqueue(...prefix, { timeoutMs: 200 });
enqueue.call(null, ...prefix, { timeoutMs: 200 });
enqueue.apply(null, [...prefix, { timeoutMs: 200 }]);
enqueue.apply(null, prefix);
enqueue("sample", { timeoutMs: 200 }, ...prefix);
enqueue.call(null, "sample", { timeoutMs: 200 }, ...prefix);
enqueue.apply(null, ["sample", { timeoutMs: 200 }, ...prefix]);

function unrelated(enqueue: (...args: unknown[]) => void) {
  enqueue(...prefix, { timeoutMs: 200 });
  enqueue.call(null, ...prefix, { timeoutMs: 200 });
  enqueue.apply(null, [...prefix, { timeoutMs: 200 }]);
}
