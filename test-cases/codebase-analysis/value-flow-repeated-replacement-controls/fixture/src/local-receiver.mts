import { tracked, rhs } from "@vendor/local-receiver-controls";
import { local } from "./local-object.mjs";
declare const key: string;
// Repository object exports have no supported replacement receiver identity.
local.get = () => 2;
local[key] = () => 3;
for (const item of ["a", "b"]) {
  tracked.get(item);
  tracked.get = rhs.get;
}
local.get();
