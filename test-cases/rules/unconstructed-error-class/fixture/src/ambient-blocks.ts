// Not flagged: `declare` describes code outside the analyzed source, exactly
// like a declaration file, even inside an ordinary `.ts` file.
export declare class DeclaredError extends Error {}

declare module "virtual-errors" {
  // Not flagged: an ambient module block declares an external module's class.
  export class ModuleBlockError extends Error {}
}
