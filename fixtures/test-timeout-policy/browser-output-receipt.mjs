import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
/** Read only after the SDK process closes and its owned group drains. */
export async function readBrowserOutputs(directory) {
    const read = (name) => readFile(join(directory, name), 'utf8').catch(() => '');
    return {
        routing: await read('routing.jsonl'),
        optimizer: await read('optimizer.jsonl'),
        reporters: await read('reporters.json'),
        browserConsole: await read('browser-console.log'),
    };
}
