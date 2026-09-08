import { parseDate } from "./utils.mts";

export function shadowedImportedCaller(input: string) {
  const parseDate = (value: string) => new Date(value);
  return parseDate(input);
}
