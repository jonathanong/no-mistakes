import { it, vi } from 'vitest';
import { readdirSync } from 'node:fs';
function direct() { return readdirSync(process.env.DIRECT); }
function coalesced() { return readdirSync(process.env.COALESCED ?? 'fixtures'); }
function fallback() { return readdirSync(process.env.FALLBACK || 'fixtures'); }
function parameter(value) { return readdirSync(process.env.PARAMETER); }
function rest(...values) { return readdirSync(process.env.REST); }
function* generator() { return readdirSync(process.env.GENERATOR); }
function multiple() { setup(); return readdirSync(process.env.MULTIPLE); }
const concise = () => readdirSync(process.env.CONCISE);
const blocked = () => { return readdirSync(process.env.BLOCKED); };
function notReturned() { readdirSync(process.env.NOT_RETURNED); }
function noArgument() { return readdirSync(); }
function otherRoot() { return readdirSync('fixtures'); }
it('direct', () => { vi.stubEnv('A', 'fixtures'); directRead(); });
it('function', function() { vi.stubEnv('B', 'fixtures'); functionRead(); });
it('concise', () => conciseRead());
it('spread', ...callbacks);
it('noncallback', 42);
it('interrupted', () => { vi.stubEnv('C', 'fixtures'); const value = 1; interruptedRead(); });
it('arguments', () => { vi.stubEnv('D', 'fixtures'); argumentRead('fixtures'); });
it('member', () => { vi.stubEnv('E', 'fixtures'); object.memberRead(); });
it('different-method', () => { vi.stubGlobal('F', 'fixtures'); wrongMethodRead(); });
it('wrong-object', () => { other.stubEnv('G', 'fixtures'); wrongObjectRead(); });
it('computed-method', () => { vi['stubEnv']('H', 'fixtures'); computedRead(); });
it('write-not-call', () => { value = 'fixtures'; assignmentRead(); });
it('read-not-call', () => { vi.stubEnv('I', 'fixtures'); value; });
it('shadowed', () => { const vi = other; vi.stubEnv('J', 'fixtures'); shadowedRead(); });
