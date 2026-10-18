const { createHash } = require("node:crypto");
const { execFileSync } = require("node:child_process");
const { readFileSync, readdirSync, lstatSync, appendFileSync } = require("node:fs");
const { join, resolve } = require("node:path");
const records = require("./hosted-check-reusable-builds.json");

const FILES = ["SHA256SUMS", "cpu-probe", "no-mistakes", "provenance.json"];
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

function select(runId, variant, sourceCommit, manifest = records) {
  if (!/^[1-9][0-9]*$/.test(String(runId)) || !Object.hasOwn(manifest, String(runId)))
    throw new Error("Artifact run is not explicitly registered for reuse");
  if (!["baseline", "current"].includes(variant)) throw new Error("Invalid artifact variant");
  const record = manifest[String(runId)][variant];
  if (!record || !/^[0-9a-f]{40}$/.test(sourceCommit) || record.sourceCommit !== sourceCommit)
    throw new Error("Reused artifact source does not match exact requested source");
  if (!Number.isSafeInteger(record.artifactId) || record.artifactId <= 0)
    throw new Error("Invalid registered artifact ID");
  if (JSON.stringify(Object.keys(record.files).sort()) !== JSON.stringify(FILES))
    throw new Error("Registered artifact must hash all four production files");
  if (!Object.values(record.files).every((hash) => /^[0-9a-f]{64}$/.test(hash)))
    throw new Error("Invalid registered artifact checksum");
  return record;
}

function verify(runId, variant, sourceCommit, directory, source, observed, manifest = records) {
  const record = select(runId, variant, sourceCommit, manifest);
  if (JSON.stringify(readdirSync(directory).sort()) !== JSON.stringify(FILES))
    throw new Error("Reused artifact has missing or extra production files");
  for (const file of FILES) {
    const path = join(directory, file);
    if (!lstatSync(path).isFile() || digest(readFileSync(path)) !== record.files[file])
      throw new Error(`Reused artifact hash mismatch: ${file}`);
  }
  const build = JSON.parse(readFileSync(join(directory, "provenance.json"), "utf8"));
  if (
    build.commit !== sourceCommit ||
    observed.commit !== sourceCommit ||
    build.tree !== observed.tree
  )
    throw new Error("Reused build does not match checked-out source identity");
  const original = readFileSync(join(source, "crates/no-mistakes/Cargo.toml"), "utf8");
  const crateType = 'crate-type = ["rlib"]';
  if (!original.includes(crateType)) throw new Error("Unexpected source crate type");
  const effective = original.replace(crateType, 'crate-type = ["rlib", "cdylib"]');
  if (
    build.sourceVersion !== original.trim() ||
    build.cargoLockSha256 !== digest(readFileSync(join(source, "Cargo.lock"))) ||
    build.sourceManifestSha256 !== digest(original) ||
    build.effectiveBuildManifestSha256 !== digest(effective) ||
    build.releaseManifestPatch !== "rlib plus cdylib, identical to release.yml"
  )
    throw new Error("Reused build Cargo inputs do not match source checkout");
  const checksums = readFileSync(join(directory, "SHA256SUMS"), "utf8");
  if (
    build.binarySha256 !== record.files["no-mistakes"] ||
    checksums !== `${record.files["no-mistakes"]}  production-cli/no-mistakes\n`
  )
    throw new Error("Reused binary checksum does not match build provenance");
  return {
    reused: true,
    originalRunId: String(runId),
    originalArtifactId: record.artifactId,
    variant,
    sourceCommit,
    sourceTree: observed.tree,
    files: record.files,
    compiler: build.rustc,
    command: build.command,
    note: "Original production bytes/provenance retained; no compiler installation or rebuild.",
  };
}

if (require.main === module) {
  try {
    const [operation, runId, variant, sourceCommit, directory, source] = process.argv.slice(2);
    if (operation === "select") {
      const record = select(runId, variant, sourceCommit);
      if (!process.env.GITHUB_OUTPUT) throw new Error("Missing workflow output destination");
      appendFileSync(process.env.GITHUB_OUTPUT, `artifact-id=${record.artifactId}\n`);
    } else if (operation === "verify") {
      const checkout = resolve(source);
      const git = (...args) =>
        execFileSync("git", args, { cwd: checkout, encoding: "utf8" }).trim();
      git("diff", "--exit-code", "HEAD");
      console.log(
        JSON.stringify(
          verify(runId, variant, sourceCommit, resolve(directory), checkout, {
            commit: git("rev-parse", "HEAD"),
            tree: git("rev-parse", "HEAD^{tree}"),
          }),
          null,
          2,
        ),
      );
    } else throw new Error("Expected select or verify operation");
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { select, verify };
