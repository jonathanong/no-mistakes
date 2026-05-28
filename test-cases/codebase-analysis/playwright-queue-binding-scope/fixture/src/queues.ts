// Two queue bindings sharing a job name. The binding-scoped hint key
// keeps `emailQueue.add("sync")` from being conflated with
// `billingQueue.add("sync")` on the dependent side.
export interface QueueLike<T> {
  add(name: string, data: T): Promise<void>;
}

export const emailQueue: QueueLike<{ userId: string }> = {
  async add(_name: string, _data: { userId: string }) {},
};

export const billingQueue: QueueLike<{ userId: string }> = {
  async add(_name: string, _data: { userId: string }) {},
};
