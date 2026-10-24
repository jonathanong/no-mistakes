import { read } from "@example/db";
import type { Options } from "./options-escape-helpers";
import { importedAwait, importedYield } from "./options-await-assertion-helpers";

async function localAwait(options: Options) { (await options.query!)("BEGIN"); }
function* localYield(options: Options) { (yield options.query!)("BEGIN"); }

localAwait({ query: read, limit: 1 });
localYield({ query: read, limit: 1 });
importedAwait({ query: read, limit: 1 });
importedYield({ query: read, limit: 1 });
