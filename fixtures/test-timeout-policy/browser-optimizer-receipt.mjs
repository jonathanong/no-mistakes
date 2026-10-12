import { expect } from 'vitest';
/** Assert actual served client-module receipts, not a config-hook forecast. */
export function assertClientOptimizerClosure(raw, owners, module) {
    const records = raw
        .trim()
        .split('\n')
        .filter(Boolean)
        .map((line) => JSON.parse(line));
    expect(records.some((record) => record.phase === 'client-transform' && record.moduleId.includes(module))).toBe(true);
    for (const record of records) {
        expect(record.phase).toBe('client-transform');
        for (const optimizer of [record.client, record.clientDeclared, record.activeEnvironment]) {
            expect(optimizer).toBeDefined();
            expect(optimizer.include).toContain('vitest-extra');
            for (const owner of owners)
                expect(optimizer.exclude).toContain(owner);
            for (const include of optimizer.include ?? []) {
                for (const token of include.split('>').map((part) => part.trim())) {
                    for (const owner of owners) {
                        expect(token === owner || token.startsWith(owner + '/'), `client include bypass: ${include}`).toBe(false);
                    }
                }
            }
        }
    }
    return records;
}
