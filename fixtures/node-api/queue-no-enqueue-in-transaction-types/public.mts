import type {
  QueueNoEnqueueInTransactionBoundary,
  QueueNoEnqueueInTransactionOptions,
  QueueNoEnqueueInTransactionPublisher,
} from "no-mistakes";

const transaction: QueueNoEnqueueInTransactionBoundary = {
  module: "@sample/database",
  export: "transaction",
  callbackArgument: 1,
};
const publisher: QueueNoEnqueueInTransactionPublisher = {
  module: "@sample/jobs",
  export: "enqueue",
};

export const options: QueueNoEnqueueInTransactionOptions = {
  transactions: [transaction],
  publishers: [publisher],
};

export const advised: QueueNoEnqueueInTransactionOptions = {
  transactions: [transaction],
  publishers: [publisher],
  unanalyzable: "advise",
};
