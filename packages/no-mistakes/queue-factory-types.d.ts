/** Select a positional argument or one static property of that argument. */
export type QueueFactoryArgument = number | QueueFactoryPropertyArgument;

export interface QueueFactoryPropertyArgument {
  arg: number;
  property: string;
}

export interface QueueEnqueueFactoryBase {
  callee: string;
  job: QueueFactoryArgument;
  /** Options on the factory call itself. */
  options?: QueueFactoryArgument;
  /** Default job options selected from the factory call. */
  defaults?: QueueFactoryArgument;
  /** Options on a call through the factory's immutable returned callable. */
  optionsOnReturnedCall?: number;
}

export type QueueEnqueueFactoryOptions = QueueEnqueueFactoryBase &
  ({ queue: QueueFactoryArgument; queueName?: never } | { queueName: string; queue?: never });

export interface QueueNameFactoryOptions {
  callee: string;
  name?: number;
}

export interface QueueWorkerFactoryOptions {
  callee: string;
  name: number;
  processor: number;
}

export interface QueueJobSchedulerFactoryOptions {
  callee: string;
  template: number;
}

/** Configuration for the `queues.factories` object in the selected config file. */
export interface QueueFactoriesOptions {
  queue?: Array<string | QueueNameFactoryOptions>;
  enqueue?: QueueEnqueueFactoryOptions[];
  worker?: QueueWorkerFactoryOptions[];
  jobScheduler?: QueueJobSchedulerFactoryOptions[];
  enqueueAndWait?: string[];
  enqueueAndWaitPatterns?: string[];
  forbidEnqueueAndWaitDeclarations?: boolean;
}

/** The legacy list and explicit role forms of `queues.factories`. */
export type QueueFactories = string[] | QueueFactoriesOptions;
