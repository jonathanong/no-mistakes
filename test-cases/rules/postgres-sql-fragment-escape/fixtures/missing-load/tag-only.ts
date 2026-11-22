import sql from "@example/sql";

declare const target: string;

export async function load() {
  require(target);
}
