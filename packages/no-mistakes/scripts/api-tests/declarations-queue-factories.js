const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("queue factory configuration selectors are public and exported", () => {
  const types = readFileSync(join(packageRoot, "queue-factory-types.d.ts"), "utf8");
  for (const name of [
    "QueueFactoryArgument",
    "QueueFactoryPropertyArgument",
    "QueueEnqueueFactoryBase",
    "QueueEnqueueFactoryOptions",
    "QueueFactoriesOptions",
    "QueueFactories",
    "QueueJobRetentionKey",
    "QueueUnboundedJobRetentionOptions",
    "QueueUnboundedJobRetentionConfig",
  ]) {
    assert.match(types, new RegExp(`export (?:type|interface) ${name}\\b`));
  }
  assert.match(types, /arg: number;/);
  assert.match(types, /property: string;/);
  assert.match(types, /job: QueueFactoryArgument;/);
  assert.match(types, /queue: QueueFactoryArgument; queueName\?: never/);
  assert.match(types, /queueName: string; queue\?: never/);
  assert.match(types, /defaults\?: QueueFactoryArgument;/);
  assert.match(types, /optionsOnReturnedCall\?: number;/);
  assert.match(types, /options\?: QueueFactoryArgument;/);
  for (const file of ["types.d.ts", "index.d.ts"]) {
    assert.match(
      readFileSync(join(packageRoot, file), "utf8"),
      /export type \* from "\.\/queue-factory-types";/,
    );
  }
});
