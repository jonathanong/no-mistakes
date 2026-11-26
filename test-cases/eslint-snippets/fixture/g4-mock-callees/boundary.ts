import { vi } from "vitest";
declare const specifier: string;
const factory = () => ({ widget: () => 1 });

vi.mock(specifier, factory);
(vi.mock as typeof vi.mock)(specifier, factory);
vi.mock!(specifier, factory);
(vi.mock satisfies typeof vi.mock)(specifier, factory);
(vi.mock as typeof vi.mock).call(vi, specifier, factory);
((0, vi.mock)! as typeof vi.mock)(specifier, factory);

function useLocal(vi: { mock: (name: string, factory: () => object) => void }) {
  (vi.mock as typeof vi.mock)(specifier, factory);
}
