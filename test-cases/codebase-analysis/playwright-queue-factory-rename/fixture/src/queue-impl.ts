export class Queue<T> {
  constructor(public readonly name: string) {}
  async add(_name: string, _data: T): Promise<void> {}
}
