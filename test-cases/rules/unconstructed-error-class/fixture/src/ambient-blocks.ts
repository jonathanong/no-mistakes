// Not flagged: `declare` describes code outside the analyzed source, exactly
// like a declaration file, even inside an ordinary `.ts` file.
export declare class DeclaredError extends Error {}

declare module "virtual-errors" {
  // Not flagged: an ambient module block declares an external module's class.
  export class ModuleBlockError extends Error {}
}

export declare namespace AmbientNamespace {
  // Not flagged: a `declare namespace` is ambient, whatever its members say.
  export class AmbientNamespaceError extends Error {}
}

declare global {
  // Not flagged: a global augmentation is ambient too.
  class GlobalAugmentationError extends Error {}
}
