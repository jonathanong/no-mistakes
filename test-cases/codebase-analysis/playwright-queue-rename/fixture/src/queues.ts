// Minimal stub so the queue imports in the fixture resolve. Real projects
// would back this with BullMQ or similar; the only thing the analyzer
// needs is a binding it can resolve, so a typed placeholder suffices.
export interface QueueLike<T> {
  add(name: string, data: T): Promise<void>;
}

export const emailQueue: QueueLike<{ userId: string }> = {
  async add(_name: string, _data: { userId: string }) {},
};
