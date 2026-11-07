export interface RecordValue {
  id: string;
  value: number;
}
export declare function readValue(id: string): Promise<RecordValue>;
export type IndexedValues = Readonly<Record<string, RecordValue>>;
