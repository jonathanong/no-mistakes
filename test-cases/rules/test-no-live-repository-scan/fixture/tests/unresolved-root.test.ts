// Advisory: the scan root is a parameter or a mutable binding, so its
// provenance cannot be resolved statically.
import { readdirSync } from "node:fs";
import { listFiles } from "../helpers/list-files";

let moving = "somewhere";
moving = process.env.SCAN_ROOT ?? moving;
readdirSync(moving);
listFiles("fixtures");
