const name = "./columns";

export async function load(extra: unknown) {
  const named = require(name);
  const templated = await import(`./${name}`);
  named.append(extra);
  templated.append(extra);
}
