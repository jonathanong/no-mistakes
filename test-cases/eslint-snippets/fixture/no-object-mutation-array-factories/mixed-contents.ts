import { row } from "./row";
export function mixed() {
  const rows = [{ id: 1 }, row];
  const [first] = rows;
  first.id = 2;
}
