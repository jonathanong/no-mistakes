const { spawnSync } = require("node:child_process");
const { readFileSync, writeFileSync } = require("node:fs");
const { basename, join, resolve } = require("node:path");
const {
  TARGET,
  STUB,
  hashFile,
  externalTarget,
  removeStubArtifacts,
  stageArtifacts,
} = require("./hosted-check-build-artifacts.cjs");
const { validateCompiler } = require("./hosted-check-refs.cjs");

const ARGS = [
  "build",
  "--release",
  "--locked",
  "--target",
  TARGET,
  "-p",
  "no-mistakes",
  "--bin",
  "no-mistakes",
];
const FLAGS = {
  CARGO_PROFILE_RELEASE_LTO: "fat",
  CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "1",
  CARGO_BUILD_JOBS: "1",
  CARGO_INCREMENTAL: "0",
  RUSTFLAGS: "-D warnings",
  NO_MISTAKES_BUILD_NAPI: "1",
};

function command(executable, args, cwd) {
  const result = spawnSync(executable, args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
    timeout: 25 * 60_000,
  });
  if (result.error || result.status !== 0)
    throw new Error(`${executable} failed: ${result.error || result.status}`);
  return result.stdout.trim();
}

function productionManifest(text) {
  const original = 'crate-type = ["rlib"]';
  if (
    text.split(original).length !== 2 ||
    !/^\s*default\s*=\s*\[[^\]]*"mermaid-validation"[^\]]*\]/m.test(text)
  ) {
    throw new Error("Expected the official release crate type and default Mermaid features");
  }
  // The measured CLI needs only rlib. Adding cdylib also links the unused addon,
  // even with --bin, and can exhaust the production job's build deadline.
  return text;
}

function verifyEnvironment(env, compiler, rustc) {
  validateCompiler(compiler);
  for (const [key, value] of Object.entries(FLAGS)) {
    if (env[key] !== value) throw new Error(`Expected production ${key}=${value}`);
  }
  if (env.RUSTUP_TOOLCHAIN !== compiler || !rustc.split("\n").includes(`release: ${compiler}`)) {
    throw new Error("Compiler differs from the resolved exact stable version");
  }
}

function build(phase, source, output, refs, variant, run = command, env = process.env) {
  if (
    !["dependencies", "production"].includes(phase) ||
    !["baseline", "candidate"].includes(variant)
  ) {
    throw new Error("Expected a dependency/production phase and baseline/candidate variant");
  }
  const selected = refs[variant];
  const target = externalTarget(source, env.CARGO_TARGET_DIR);
  const commit = run("git", ["rev-parse", "HEAD"], source);
  const tree = run("git", ["rev-parse", "HEAD^{tree}"], source);
  if (commit !== selected.commit || tree !== selected.tree)
    throw new Error("Checkout differs from resolved source identity");
  run("git", ["diff", "--exit-code", "HEAD"], source);
  const rustc = run("rustc", ["-Vv"], source);
  verifyEnvironment(env, refs.compiler, rustc);
  const manifest = join(source, "crates/no-mistakes/Cargo.toml");
  const original = productionManifest(readFileSync(manifest, "utf8"));
  const entrypoints = ["lib.rs", "main.rs"].map((name) =>
    join(source, "crates/no-mistakes/src", name),
  );
  const sourceHashes = Object.fromEntries(
    entrypoints.map((path) => {
      if (readFileSync(path, "utf8").includes(STUB.trim()))
        throw new Error("Dependency stub cannot be a production source");
      return [basename(path), hashFile(path)];
    }),
  );
  const cargo = run("cargo", ["-V"], source);
  const sourceManifestSha256 = require("node:crypto")
    .createHash("sha256")
    .update(original)
    .digest("hex");
  const dependencyIdentity = {
    variant,
    commit,
    tree,
    compiler: refs.compiler,
    rustc,
    cargo,
    sourceHashes,
    sourceManifestSha256,
    flags: FLAGS,
    buildArgs: ARGS,
  };
  const dependencyRecord = join(target, "hosted-dependency-identity.json");
  if (phase === "dependencies") {
    // Preserve the real dependency graph, feature set, build.rs, and release flags.
    writeFileSync(entrypoints[0], STUB);
    writeFileSync(entrypoints[1], `${STUB}fn main() {}\n`);
    run("cargo", ARGS, source);
    writeFileSync(dependencyRecord, JSON.stringify(dependencyIdentity));
    return;
  }
  // Restored dependency artifacts must never let Cargo reuse the placeholder CLI.
  if (
    JSON.stringify(JSON.parse(readFileSync(dependencyRecord, "utf8"))) !==
    JSON.stringify(dependencyIdentity)
  ) {
    throw new Error("Restored dependencies differ from the resolved source/compiler/build flags");
  }
  const removedStubArtifacts = removeStubArtifacts(target);
  if (removedStubArtifacts === 0)
    throw new Error("Restored dependencies did not contain the placeholder package");
  run("cargo", ARGS, source);
  const changed = run("git", ["diff", "--name-only", "HEAD"], source);
  if (changed !== "" || readFileSync(manifest, "utf8") !== original) {
    throw new Error("Production build changed tracked sources");
  }
  stageArtifacts(
    source,
    target,
    output,
    {
      schemaVersion: 1,
      variant,
      refs,
      commit,
      tree,
      compiler: refs.compiler,
      rustc,
      cargo,
      sourceVersion: original.match(/^version = "([^"]+)"/m)?.[1],
      sourceManifestSha256,
      sourceHashes,
      removedStubArtifacts,
      command: `NO_MISTAKES_BUILD_NAPI=1 cargo ${ARGS.join(" ")}`,
      features: "default (including mermaid-validation)",
      lto: "fat",
      codegenUnits: 1,
      buildJobs: 1,
      rustflags: env.RUSTFLAGS,
      incremental: false,
      target: TARGET,
      imageOS: env.ImageOS,
      imageVersion: env.ImageVersion,
      releaseManifestPatch: null,
      dependencyPreparation:
        "Temporary entrypoint stubs; real package artifacts removed before production compile",
    },
    run,
  );
}

if (require.main === module) {
  const [phase, source, output, refsPath, variant] = process.argv.slice(2);
  build(
    phase,
    resolve(source),
    resolve(output),
    JSON.parse(readFileSync(refsPath, "utf8")),
    variant,
  );
}
module.exports = { ARGS, FLAGS, productionManifest, verifyEnvironment, build };
