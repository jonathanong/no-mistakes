import { read } from "db";
import { importedStatement } from "./producer";

export async function imported() {
  const q = importedStatement();
  return read(q.text, q.values);
}
