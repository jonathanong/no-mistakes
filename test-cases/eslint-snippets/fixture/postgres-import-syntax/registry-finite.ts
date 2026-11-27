import { read } from "./db";

const COLS = { id: "id", name: "name" };

function lookup(k: string) {
  read(`SELECT ${COLS[k as keyof typeof COLS]}`);
  read(`SELECT ${COLS[k satisfies keyof typeof COLS]}`);
  read(`SELECT ${COLS[<keyof typeof COLS>k]}`);
  read(`SELECT ${COLS[(k as keyof typeof COLS)!]}`);
}
