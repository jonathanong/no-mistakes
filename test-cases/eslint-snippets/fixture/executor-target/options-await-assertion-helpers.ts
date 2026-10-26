import type { Options } from "./options-escape-helpers";

export async function importedAwait(options: Options) { (await options.query!)("BEGIN"); }
export function* importedYield(options: Options) { (yield options.query!)("BEGIN"); }
