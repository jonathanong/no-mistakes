import sql from "@example/sql";
import {missing} from "./missing";

declare function consume(value: unknown): void;

export const columns = sql`code`;
consume(missing);
