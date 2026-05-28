export interface QueueLike<T> {
  add(name: string, data: T): Promise<void>;
  addBulk(jobs: Array<{ name: string; data: T }>): Promise<void>;
}

export const emailQueue: QueueLike<{ userId: string }> = {
  async add(_name: string, _data: { userId: string }) {},
  async addBulk(_jobs: Array<{ name: string; data: { userId: string } }>) {},
};
