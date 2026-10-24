import { query } from "@example/db";

export function findUnconfiguredView() {
  return query("SELECT 1 FROM public.unconfigured_view");
}
