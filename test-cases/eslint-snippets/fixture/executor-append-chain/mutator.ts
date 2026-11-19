import { MUTATED } from "./fragments";

export function extend() {
  return MUTATED.append(sql`, extra`);
}

declare function sql(strings: TemplateStringsArray): never;
