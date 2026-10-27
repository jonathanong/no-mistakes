const before = Map;

// Erased mapped/inferred names must not hide the runtime global after the type.
type OptionalReadonly<Value> = { readonly [Map in keyof Value]?: Value[Map] };
type Mutable<Value> = { -readonly [Map in keyof Value]-?: Value[Map] };
type ExplicitPlus<Value> = { +readonly [Map in keyof Value]+?: Value[Map] };
type Inferred<Value> = Value extends Promise<infer Map> ? Map : Value;
type Pair = [string, unknown];
type Imported = import("./multi-hop/result").TaskResult;
type GenericImported = import("./multi-hop/result").Result<string>;

const after = Map;
