import { tables } from "./tables";

export function write(key: string) {
  tables[key] = "other.users";
}
