import { query } from "@app/db";
function expensive(value: string) {
  if (unknownA) value += " a";
  if (unknownB) value += " b";
  if (unknownC) value += " c";
  if (unknownD) value += " d";
  return value;
}
query(expensive("SELECT 2"));
