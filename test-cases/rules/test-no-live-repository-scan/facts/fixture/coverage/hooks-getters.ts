import { vi, afterEach, "beforeEach" as prepare, type beforeAll as typedHook } from "vitest";
prepare(() => { process.env.QUOTED = "./fixture"; });
typedHook(() => { process.env.TYPED = "./fixture"; });
afterEach(function cleanup() { vi.unstubAllEnvs(); });
function namedCleanup() { vi.unstubAllEnvs(); }
afterEach(namedCleanup);
other.unstubAllEnvs();
const escapedReset = vi.unstubAllEnvs;
function emptyReturn() { return; }
function unknownFallback() { return process.env.X ?? dynamicFallback(); }
function andFallback() { return process.env.X && "./fixture"; }
function validFallback() { return process.env.X || "./fixture"; }
