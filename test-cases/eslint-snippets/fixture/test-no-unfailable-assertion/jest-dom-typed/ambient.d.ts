interface JestDomMatchers {
  not: JestDomMatchers;
  resolves: JestDomMatchers;
  toBeDefined(): void;
  toBeNull(): void;
  toBeInTheDocument(): void;
}

declare module "vitest" {
  export function expect(actual: unknown): JestDomMatchers;
}

declare module "@testing-library/react" {
  export const screen: {
    getByText(text: string): HTMLElement;
    getAllByRole(role: string): HTMLElement[];
  };
}
