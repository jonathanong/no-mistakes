import { registry } from "./barrel";
import { TABLE_NAMES } from "./catalog";
import type { TableName } from "./catalog";

export const selected = registry;
export const names = TABLE_NAMES;
export type ConsumerTable = TableName;
