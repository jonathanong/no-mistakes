import { read } from "db";
import { getRelationTableNameOrThrow } from "./barrel.ts";

const unused = getRelationTableNameOrThrow({ subject: "topic", predicate: "parent", object: "topic" });
export const list = () => read("SELECT id FROM items");
