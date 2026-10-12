export declare function assertTestTimeout(value: unknown, label: string, maximum?: number): number;
export declare function assertTimeoutConfig(config: object, maximum?: number): void;
/** Preserve SDK option/property semantics while capturing one validated deadline. */
export declare function snapshotTimeoutOptions(
  options: object,
  label: string,
  maximum?: number,
  nullInherits?: boolean,
): object;
/** Install before collection, composing with the existing runner's config object. */
export declare function protectTimeoutConfig(config: object, maximum?: number): void;
