import {
  resolveConfig,
  type ConfigResolveOptions,
  type ConfigResolveView,
  type DeclaredConfig,
  type DeclaredRule,
  type DeclaredConfigValue,
  type DeclaredConfigObject,
  type ResolvedConfig,
} from "no-mistakes";

const declared: DeclaredConfig = await resolveConfig({ root: ".", view: "declared" });
const rule: DeclaredRule | undefined = declared.rules?.find(
  (entry) => entry.rule === "rust-max-lines-per-file" && entry.name === "source-policy",
);
const options: DeclaredConfigValue | undefined = rule?.options;
const object: DeclaredConfigObject = { enabled: true, nested: [null, "value", 3] };
const resolved: ResolvedConfig = await resolveConfig({ root: "." });
const explicit: ResolvedConfig = await resolveConfig({ view: "resolved" });
const view: ConfigResolveView = Math.random() > 0.5 ? "declared" : "resolved";
const request: ConfigResolveOptions = { root: ".", view };
const either: DeclaredConfig | ResolvedConfig = await resolveConfig(request);
void [options, object, resolved, explicit, either];
