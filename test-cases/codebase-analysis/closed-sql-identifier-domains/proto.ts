const PROTO_OBJECT = { "__proto__": "users" };
const PROTO_METADATA = [{ "__proto__": "users", table: "users" }];
const ORDINARY_MAP = new Map([["__proto__", "users"]]);
const table = ORDINARY_MAP.get("__proto__");
