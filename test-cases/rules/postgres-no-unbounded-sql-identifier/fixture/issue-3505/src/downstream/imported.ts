import { read } from "db";
import { importedStatement } from "./producer";
import * as statements from "./producer";

export async function imported() {
  const q = importedStatement();
  return read(q.text, q.values);
}

export async function namespacedStructural() {
  return read(`/* namespace structural */ ${statements.structuralFragment.text}`, statements.structuralFragment.values);
}
