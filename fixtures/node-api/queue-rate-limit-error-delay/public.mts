import type {
  QueueRateLimitErrorDelayOptions,
  QueueRateLimitErrorDelayTarget,
} from "no-mistakes";

const target: QueueRateLimitErrorDelayTarget = {
  module: "glide-mq",
  export: "RateLimitError",
  delayArgument: 0,
};

const objectTarget: QueueRateLimitErrorDelayTarget = {
  module: "./src/errors",
  export: "ProviderRateLimitError",
  delayArgument: 1,
  delayProperty: "retryAfterMs",
};

const options: QueueRateLimitErrorDelayOptions = {
  targets: [target, objectTarget],
  unanalyzable: "advise",
};

void options;
