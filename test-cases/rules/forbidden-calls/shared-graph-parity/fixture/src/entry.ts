import { Boom, danger } from "@fixture/lib";
import { localDanger } from "./local";

export function run() {
  danger();
  localDanger();
  return new Boom("boom");
}
