import type {
  QueueNoEnqueueInTransactionBegin,
  QueueNoEnqueueInTransactionBoundary,
  QueueNoEnqueueInTransactionOptions,
  QueueNoEnqueueInTransactionPublisher,
} from "no-mistakes";

const transaction: QueueNoEnqueueInTransactionBoundary = {
  module: "@sample/database",
  export: "transaction",
  callbackArgument: 1,
};
const beginTransaction: QueueNoEnqueueInTransactionBegin = {
  module: "@sample/database",
  export: "beginTransaction",
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

export const scopedOptions: QueueNoEnqueueInTransactionOptions = {
  beginTransactions: [beginTransaction],
  publishers: [publisher],
};

export const mixedOptions: QueueNoEnqueueInTransactionOptions = {
  transactions: [transaction],
  beginTransactions: [beginTransaction],
  publishers: [publisher],
};
