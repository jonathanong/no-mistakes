export type Representation<Value> = Value extends Date
  ? string
  : Value extends ReadonlyArray<infer Item>
    ? Representation<Item>[]
    : Value extends Record<string, unknown>
      ? { [Key in keyof Value]: Representation<Value[Key]> }
      : Value;
