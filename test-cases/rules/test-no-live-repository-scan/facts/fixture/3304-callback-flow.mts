import { mkdtempSync, readdirSync, rmSync } from 'node:fs';
function withRoot(callback) {
  const root = mkdtempSync('flow-');
  try { callback(root); } finally { rmSync(root, { recursive: true }); }
}
async function withAwaitedRoot(callback) {
  const root = mkdtempSync('flow-');
  try { await callback(root); } finally { rmSync(root, { recursive: true }); }
}
function withoutArgument(callback) {
  const root = mkdtempSync('flow-');
  try { callback(); } finally { rmSync(root, { recursive: true }); }
}
function withoutParameter() {}
function unusedParameter(callback) { type Callback = typeof callback; }
const arrowRoot = (callback) => {
  const root = mkdtempSync('flow-');
  try { callback(root); } finally { rmSync(root, { recursive: true }); }
};
withRoot((root) => { readdirSync(root, 'sync-block'); });
withRoot((root) => readdirSync(root, 'unawaited-concise'));
withRoot(async (root) => { readdirSync(root, 'unawaited-async'); });
withAwaitedRoot((root) => readdirSync(root, 'awaited-concise'));
withAwaitedRoot(async function(root) { readdirSync(root, 'awaited-function'); });
withoutArgument((root) => { readdirSync(root, 'missing-argument'); });
withoutParameter((root) => { readdirSync(root, 'missing-parameter'); });
unusedParameter((root) => { readdirSync(root, 'unused-parameter'); });
arrowRoot((root) => { readdirSync(root, 'arrow-helper'); });
