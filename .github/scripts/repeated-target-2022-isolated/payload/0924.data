export type Protector<T extends { config: object } = { config: object }> = (
  runner: T,
  maximum: number,
) => void;
export type RunnerConstructor = new (...args: never[]) => {
  config: object;
};
/** Compose a final public constructor while retaining direct and derived new.target contracts. */
export declare function guardPublicRunner<T extends RunnerConstructor>(
  original: T,
  maximum: number,
  protect: Protector<InstanceType<T>>,
): T;
