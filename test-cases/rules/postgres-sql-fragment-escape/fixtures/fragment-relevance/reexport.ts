import {missing} from "./missing";

declare function consume(value: unknown): void;

export {columns as exposedColumns} from "./columns";
consume(missing);
