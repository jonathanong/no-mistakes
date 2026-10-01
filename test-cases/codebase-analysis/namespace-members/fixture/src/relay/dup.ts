// Two modules export a namespace of the same name through `export *`, so which
// one `Shared` means is ambiguous. A package outside the repository holds none.
export * from "./dup-one";
export * from "./dup-two";
export * from "some-package";
