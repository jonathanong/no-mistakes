import { parseDate as impl } from "./utils.mts";

export const parsePrivate = impl;

export function renderPrivateAliasDate(input: string) {
  return impl(input).toISOString();
}
