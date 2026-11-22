import { COLS } from "./columns";
import "./related-broken";

declare const moduleName: string;

export async function read() {
  const requested = await import(moduleName);
  return { requested, usesColumns: typeof COLS === "object" };
}
