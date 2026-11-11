import { type getLocalDir as typeOnly, getLocalDir as runtime } from "./getter-definitions.mjs";
import { readdirSync } from "node:fs";
readdirSync(typeOnly());
readdirSync(runtime());
