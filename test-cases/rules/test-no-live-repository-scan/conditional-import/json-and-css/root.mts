import { it } from "vitest";
import { scan } from "./scan.mjs";
import catalog from "./catalog.json";
import "./theme.css";

it("scan", () => {
  scan();
  return catalog.id;
});
