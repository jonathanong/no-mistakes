import { parseDate } from "./utils.mts";

export default function formatWrappedDate(value: string) {
  return parseDate(value).toISOString();
}
