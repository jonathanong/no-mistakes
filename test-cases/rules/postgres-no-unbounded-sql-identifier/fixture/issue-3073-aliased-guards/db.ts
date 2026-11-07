export interface Query {
  (text: string, values?: readonly unknown[]): Promise<{ id: string }[]>
  [Symbol.asyncDispose](): Promise<void>
}
export declare function beginTransaction(): Promise<Query>
