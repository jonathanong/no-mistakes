import {later} from "./other-missing";
import {earlier} from "./missing";

declare function consume(value: unknown): void;

const namespace = await import("./columns");
void namespace.columns;
consume(later);
consume(earlier);
