import { readFileSync } from 'node:fs';
import { URL as NodeURL, fileURLToPath } from 'node:url';

const readFromPromises = require('fs').promises.readFile;

export function loadBindingResources() {
  readFromPromises('resources/require-promises-alias.txt');
  readFileSync(new NodeURL('./resources/named-url.txt', import.meta.url));
  readFileSync(
    fileURLToPath(new NodeURL('./resources/named-file-url.txt', import.meta.url)),
  );
}

// The imported constructor must stop matching when a local binding shadows it.
export function ignoredShadowedConstructor(NodeURL: unknown) {
  readFileSync(new NodeURL('./resources/shadowed-url.txt', import.meta.url));
}
