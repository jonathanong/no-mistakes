import {unrelated} from "./missing";

declare function consume(value: unknown): void;

const {plain} = await import("./columns");
void plain;
consume(unrelated);
