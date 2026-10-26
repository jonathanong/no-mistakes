// Repeated var declarations are legal but do not prove a namespace import.
var ns;
var ns;
ns.test("unknown namespace", () => error.message === "expected");
