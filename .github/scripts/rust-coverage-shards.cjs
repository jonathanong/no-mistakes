const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const {
  closeSync,
  copyFileSync,
  existsSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  realpathSync,
  readdirSync,
  readSync,
  writeFileSync,
  unlinkSync,
} = require("node:fs");
const { basename, dirname, isAbsolute, join, relative, resolve, sep } = require("node:path");

const archiveName = "rust-tests.tar.zst";
const readJson = (file) => JSON.parse(readFileSync(file, "utf8"));
const writeJson = (file, value) => writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
function digest(file) {
  const hash = createHash("sha256");
  const descriptor = openSync(file, "r");
  const buffer = Buffer.alloc(1024 * 1024);
  try {
    let count;
    while ((count = readSync(descriptor, buffer)) > 0) hash.update(buffer.subarray(0, count));
    return hash.digest("hex");
  } finally {
    closeSync(descriptor);
  }
}
function scope(env) {
  const workspace = env.GITHUB_WORKSPACE;
  const target = env.CARGO_LLVM_COV_TARGET_DIR;
  if (!workspace || !target || !env.GITHUB_SHA || basename(target) !== "target")
    throw new Error(
      "Set GITHUB_WORKSPACE, GITHUB_SHA and CARGO_LLVM_COV_TARGET_DIR ending in /target",
    );
  return { workspace: resolve(workspace), target: resolve(target), source: env.GITHUB_SHA };
}
function command(args, env) {
  const result = spawnSync("cargo", args, { env, stdio: "inherit" });
  if (result.error) throw result.error;
  return result.status ?? 1;
}
function checked(args, env, run) {
  const status = run(args, env);
  if (status !== 0) throw new Error(`cargo ${args.join(" ")} exited ${status}`);
}
function partition(index, total) {
  if (!/^[1-9]\d*$/.test(String(index)) || !/^[1-9]\d*$/.test(String(total)) || +index > +total)
    throw new Error("Expected a shard index and total with 1 <= index <= total");
  return `hash:${index}/${total}`;
}
function archive(directory, env = process.env, run = command) {
  const identity = scope(env);
  if (resolve(env.CARGO_TARGET_DIR || ".") !== identity.target)
    throw new Error("Source cargo llvm-cov show-env --sh before archiving tests");
  mkdirSync(directory, { recursive: true });
  if (existsSync(identity.target)) {
    for (const name of readdirSync(identity.target).filter((name) =>
      /\.prof(raw|data)$/.test(name),
    )) {
      const file = join(identity.target, name);
      if (!lstatSync(file).isFile()) throw new Error(`Refusing nonregular profile: ${file}`);
      unlinkSync(file);
    }
  }
  checked(
    [
      "nextest",
      "archive",
      "--locked",
      "--workspace",
      "--all-features",
      "--lib",
      "--bins",
      "--tests",
      "--archive-file",
      join(directory, archiveName),
    ],
    env,
    run,
  );
  writeJson(join(directory, "build.json"), {
    ...identity,
    archiveSha256: digest(join(directory, archiveName)),
  });
}
function verify(directory, env = process.env) {
  const identity = scope(env);
  const build = readJson(join(directory, "build.json"));
  for (const key of ["source", "workspace", "target"])
    if (identity[key] !== build[key])
      throw new Error(`Coverage archive ${key} mismatch: ${build[key]} != ${identity[key]}`);
  if (digest(join(directory, archiveName)) !== build.archiveSha256)
    throw new Error("Coverage archive SHA-256 mismatch");
  return build;
}
function shard(directory, output, index, total, env = process.env, run = command) {
  const bucket = partition(index, total);
  const build = verify(directory, env);
  if (existsSync(output)) throw new Error(`Refusing to reuse shard output: ${output}`);
  mkdirSync(join(output, "profiles"), { recursive: true });
  mkdirSync(dirname(build.target), { recursive: true });
  const references = join(resolve(output), "references.txt");
  writeFileSync(references, "");
  const started = Date.now();
  let status = 1;
  writeJson(join(output, "result.json"), {
    ...build,
    index: +index,
    total: +total,
    status,
    elapsedMs: 0,
  });
  try {
    status = run(
      [
        "nextest",
        "run",
        "--archive-file",
        join(directory, archiveName),
        "--extract-to",
        dirname(build.target),
        "--workspace-remap",
        build.workspace,
        "--partition",
        bucket,
        "--no-fail-fast",
        "--retries",
        "0",
        "--ignore-default-filter",
      ],
      {
        ...env,
        CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER: `bash ${join(__dirname, "libtest-argv-runner.sh")}`,
        INSTA_SNAPSHOT_REFERENCES_FILE: references,
        LLVM_PROFILE_FILE: join(resolve(output), "profiles", "no-mistakes-%9m.profraw"),
      },
    );
  } finally {
    writeJson(join(output, "result.json"), {
      ...build,
      index: +index,
      total: +total,
      status,
      elapsedMs: Date.now() - started,
    });
  }
  return status;
}
function reportObjects(directory, metadata, env = process.env) {
  const build = verify(directory, env);
  const previous = env.LLVM_COV_FLAGS || "";
  if (/[\r\n\0]/.test(previous) || /(^|\s)(@|--?(object|instr-profile)(=|\s|$))/.test(previous))
    throw new Error(
      "Conflicting LLVM_COV_FLAGS: object/profile overrides and response files are unsupported",
    );
  if (metadata["rust-build-meta"]?.["target-directory"] !== build.target)
    throw new Error("Coverage binary metadata target differs from the archive");
  const binaries = Object.values(metadata["rust-binaries"] || {});
  if (!binaries.length) throw new Error("Coverage archive has no test binary objects");
  const canonicalTarget = realpathSync(build.target);
  const objects = [
    ...new Set(
      binaries.map((binary) => {
        const file = binary["binary-path"];
        if (typeof file !== "string" || !isAbsolute(file) || /[\x00-\x1f\x7f]/.test(file))
          throw new Error("Coverage test binary must have an absolute regular path");
        const inside = relative(canonicalTarget, realpathSync(file));
        if (
          !inside ||
          inside === ".." ||
          inside.startsWith(`..${sep}`) ||
          isAbsolute(inside) ||
          !lstatSync(file).isFile()
        )
          throw new Error(`Coverage test binary is outside the target or is not regular: ${file}`);
        return file;
      }),
    ),
  ].sort();
  const responseName = "target/llvm-cov/nextest-objects.rsp";
  const response = join(build.workspace, responseName);
  mkdirSync(dirname(response), { recursive: true });
  writeFileSync(
    response,
    `${objects.map((file) => JSON.stringify(`-object=${file}`)).join("\n")}\n`,
  );
  writeJson(join(dirname(response), "nextest-objects.json"), { objects, response: responseName });
  return `LLVM_COV_FLAGS=${[previous, `@${responseName}`].filter(Boolean).join(" ")}`;
}
function collect(directory, input, total, env = process.env) {
  partition(1, total);
  const build = verify(directory, env);
  mkdirSync(build.target, { recursive: true });
  if (readdirSync(build.target).some((name) => /\.prof(raw|data)$/.test(name)))
    throw new Error(
      "Coverage target already contains profile data; extract a fresh archive before collecting shards",
    );
  const errors = [];
  const references = [];
  const rows = [];
  const profiles = [];
  for (let index = 1; index <= +total; index++) {
    const folder = join(input, `rust-coverage-shard-${index}`);
    try {
      const result = readJson(join(folder, "result.json"));
      for (const [key, expected] of Object.entries({ ...build, index, total: +total }))
        if (result[key] !== expected) throw new Error(`${key} mismatch`);
      const raw = readdirSync(join(folder, "profiles")).sort();
      for (const name of raw) {
        const file = join(folder, "profiles", name);
        if (!name.endsWith(".profraw") || !lstatSync(file).isFile() || lstatSync(file).size === 0)
          throw new Error(`Invalid raw profile: ${name}`);
        const destination = join(build.target, `shard-${index}-${name}`);
        if (existsSync(destination)) throw new Error(`Profile already exists: ${destination}`);
        profiles.push([file, destination]);
      }
      references.push(readFileSync(join(folder, "references.txt"), "utf8"));
      rows.push(`| ${index}/${total} | ${result.status} | ${raw.length} | ${result.elapsedMs} |`);
      if (result.status !== 0) errors.push(`Shard ${index} exited ${result.status}`);
      if (raw.length === 0) errors.push(`Shard ${index} produced no raw profiles`);
    } catch (error) {
      errors.push(`Shard ${index}: ${error.message}`);
    }
  }
  for (const [source, destination] of profiles) copyFileSync(source, destination);
  writeFileSync(join(input, "references.txt"), references.join("\n"));
  const summary = `## Rust coverage shards\n\n| Shard | Exit | Profiles | Milliseconds |\n| --- | --- | --- | --- |\n${rows.join("\n")}\n\n${errors.join("\n")}\n`;
  writeFileSync(join(input, "summary.md"), summary);
  if (env.GITHUB_STEP_SUMMARY) require("node:fs").appendFileSync(env.GITHUB_STEP_SUMMARY, summary);
  if (errors.length) throw new Error(errors.join("\n"));
}
if (require.main === module) {
  try {
    const [mode, ...args] = process.argv.slice(2);
    const arity = { archive: 1, verify: 1, shard: 4, collect: 3, "report-objects": 2 };
    if (args.length !== arity[mode])
      throw new Error(
        "Usage: rust-coverage-shards.cjs archive <bundle> | verify <bundle> | shard <bundle> <output> <index> <total> | collect <bundle> <input> <total> | report-objects <bundle> <binaries-json>",
      );
    const result =
      mode === "report-objects"
        ? reportObjects(args[0], readJson(args[1]))
        : { archive, verify, shard, collect }[mode](...args);
    if (typeof result === "string") console.log(result);
    if (typeof result === "number") process.exitCode = result;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
module.exports = { archive, verify, shard, collect, partition, reportObjects };
