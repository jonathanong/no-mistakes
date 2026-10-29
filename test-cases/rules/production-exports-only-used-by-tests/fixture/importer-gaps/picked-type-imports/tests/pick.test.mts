export type One = Pick<typeof import("../src/shapes.mjs"), "square">;
export type Two = Pick<typeof import("../src/shapes.mjs"), "square" | "circle">;
