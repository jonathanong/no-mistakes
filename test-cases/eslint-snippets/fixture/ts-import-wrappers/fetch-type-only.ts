import type { fetch } from "./types";

declare const dyn: string;
fetch("/api", { method: dyn });
fetch(dyn);
