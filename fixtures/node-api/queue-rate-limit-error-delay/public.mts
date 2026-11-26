import type {
  QueueRateLimitErrorDelayOptions,
  QueueRateLimitErrorDelayTarget,
} from "no-mistakes";

const instanceTarget: QueueRateLimitErrorDelayTarget = {
  module: "glide-mq",
  export: "Worker.RateLimitError",
  instanceDelayProperty: "delayMs",
};

const positionalTarget: QueueRateLimitErrorDelayTarget = {
  module: "glide-mq",
  export: "GroupRateLimitError",
  delayArgument: 0,
};

const objectTarget: QueueRateLimitErrorDelayTarget = {
  module: "./src/errors",
  export: "ProviderRateLimitError",
  delayArgument: 1,
  delayProperty: "retryAfterMs",
};

const options: QueueRateLimitErrorDelayOptions = {
  targets: [instanceTarget, positionalTarget, objectTarget],
  unanalyzable: "advise",
};

void options;
