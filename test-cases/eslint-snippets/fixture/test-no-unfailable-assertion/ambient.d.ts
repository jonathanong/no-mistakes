interface FixtureMatchers<T = unknown> {
  not: FixtureMatchers<T>;
  resolves: FixtureMatchers<Awaited<T>>;
  rejects: FixtureMatchers<unknown>;
  toBeDefined(): void;
  toBeUndefined(): void;
  toBeTruthy(): void;
  toBeFalsy(): void;
  toBeNull(): void;
  toBe(expected: unknown): void;
  toEqual(expected: unknown): void;
  toStrictEqual(expected: unknown): void;
  toContain(expected: unknown): void;
  toHaveTextContent(expected: string): void;
  toBeVisible(): Promise<void>;
}

interface FixtureExpect {
  <T>(actual: T): FixtureMatchers<T>;
  soft<T>(actual: T): FixtureMatchers<T>;
  poll<T>(fn: () => T): FixtureMatchers<T>;
}

declare module "vitest" {
  export const expect: FixtureExpect;
}

declare module "@testing-library/react" {
  export interface Queries {
    getByText(text: string): HTMLElement;
    getAllByRole(role: string): HTMLElement[];
    getByRole(role: string): HTMLElement;
    getByLabelText(text: string): HTMLElement;
    findByText(text: string): Promise<HTMLElement>;
    findAllByRole(role: string): Promise<HTMLElement[]>;
    queryByText(text: string): HTMLElement | null;
  }
  export const screen: Queries;
  export function render(ui: unknown): Queries;
  export function within(element: HTMLElement): Queries;
}

declare module "@testing-library/dom" {
  export function getByText(container: HTMLElement, text: string): HTMLElement;
}

declare module "@testing-library/jest-dom/vitest" {}

declare module "@playwright/test" {
  export interface Locator {
    locator(selector: string): Locator;
    getByTestId(testId: string): Locator;
    getByRole(role: string): Locator;
    getByText(text: string): Locator;
    first(): Locator;
    filter(options: { hasText?: string }): Locator;
    click(): Promise<void>;
  }
  export interface FrameLocator {
    locator(selector: string): Locator;
    getByRole(role: string): Locator;
  }
  export interface Frame {
    locator(selector: string): Locator;
    getByText(text: string): Locator;
  }
  export interface Page {
    locator(selector: string): Locator;
    getByTestId(testId: string): Locator;
    getByRole(role: string): Locator;
    frameLocator(selector: string): FrameLocator;
  }
  export const expect: FixtureExpect;
}
