// Type-only import-equals and entity-name aliases are not test bindings.
import type hidden = require("vitest");
import alias = Foo.Bar;

hidden.skip("type only", () => {});
alias.skip("entity name", () => {});
alias.test.skip("entity member", () => {});
