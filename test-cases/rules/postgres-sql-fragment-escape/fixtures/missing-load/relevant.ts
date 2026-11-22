import {widgetColumns} from "./columns";

declare const target: string;

export async function load() {
  require(target);
  await import(target);
  return {ready: typeof widgetColumns === "object"};
}
