// Two modules export a namespace of the same name through `export *`, so which
// one `Shared` means is ambiguous.
export * from "./dup-one";
export * from "./dup-two";
