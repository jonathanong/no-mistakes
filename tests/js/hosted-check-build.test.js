const assert = require("node:assert/strict");
const { cpSync, existsSync, mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const {
  resolveRefs,
  validateCompiler,
  commitIdentity,
} = require("../../.github/scripts/hosted-check-refs.cjs");
const {
  ARGS,
  FLAGS,
  build,
  productionManifest,
  verifyEnvironment,
} = require("../../.github/scripts/hosted-check-build.cjs");
const {
  TARGET,
  STUB,
  externalTarget,
  hashFile,
  ownedArtifact,
} = require("../../.github/scripts/hosted-check-build-artifacts.cjs");

const fixture = join(__dirname, "../../fixtures/hosted-check/build");
const responses = JSON.parse(readFileSync(join(fixture, "github-responses.json"), "utf8"));
const compiler = "1.93.0";
const rustc = `rustc ${compiler} (saved)\nbinary: rustc\nrelease: ${compiler}\nhost: x86_64-unknown-linux-gnu`;
const repository = "jonathanong/no-mistakes";
const harness = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const refs = resolveRefs((endpoint) => responses[endpoint], compiler, repository, harness);

function materialize(runTest) {
  const directory = mkdtempSync(join(tmpdir(), "hosted-build-test-"));
  cpSync(join(fixture, "source"), join(directory, "source"), { recursive: true });
  cpSync(join(fixture, "dependency-target"), join(directory, "target"), { recursive: true });
  try {
    runTest(directory);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

function runner(directory, phase, commands = []) {
  return (executable, args) => {
    commands.push([executable, ...args]);
    if (executable === "git" && args[1] === "--name-only") return "crates/no-mistakes/Cargo.toml";
    if (executable === "git")
      return args[1] === "HEAD"
        ? refs.candidate.commit
        : args[1] === "HEAD^{tree}"
          ? refs.candidate.tree
          : "";
    if (executable === "rustc" && args[0] === "-Vv") return rustc;
    if (executable === "cargo" && args[0] === "-V") return `cargo ${compiler} (saved)`;
    if (executable === "cargo") {
      assert.deepEqual(args, ARGS);
      if (phase === "production") {
        const profile = join(directory, "target", TARGET, "release");
        assert.equal(
          existsSync(join(profile, "no-mistakes")),
          false,
          "never reuse the placeholder CLI",
        );
        assert.equal(existsSync(join(profile, ".fingerprint/no-mistakes-stub")), false);
        assert.equal(existsSync(join(profile, "deps/libno_mistakes-stub.rlib")), false);
        assert.ok(existsSync(join(profile, "deps/libnum_cpus-saved.rlib")));
        cpSync(join(fixture, "production-target"), join(directory, "target"), { recursive: true });
      }
      return "";
    }
    if (executable === "rustc") {
      cpSync(join(fixture, "cpu-probe"), args.at(-1));
      return "";
    }
    return "no-mistakes 0.94.0";
  };
}

function environment(directory) {
  return { ...FLAGS, RUSTUP_TOOLCHAIN: compiler, CARGO_TARGET_DIR: join(directory, "target") };
}

function dependencyPhase(directory) {
  const source = join(directory, "source");
  build(
    "dependencies",
    source,
    join(directory, "output"),
    refs,
    "candidate",
    runner(directory, "dependencies"),
    environment(directory),
  );
  assert.ok(readFileSync(join(source, "crates/no-mistakes/src/main.rs"), "utf8").includes(STUB));
  // Production runs in a fresh immutable checkout, while only the target crosses jobs.
  cpSync(join(fixture, "source"), source, { recursive: true });
}

test("resolves current main and GitHub's latest stable tag once, ignoring release target_commitish", () => {
  const calls = [];
  const result = resolveRefs(
    (endpoint) => {
      calls.push(endpoint);
      return responses[endpoint];
    },
    compiler,
    repository,
    harness,
  );
  assert.deepEqual(calls, Object.keys(responses));
  assert.equal(result.candidate.ref, "refs/heads/main");
  assert.equal(result.candidate.commit, responses[calls[0]].sha);
  assert.equal(result.baseline.ref, "refs/tags/v0.94.0");
  assert.equal(result.baseline.commit, responses[calls[2]].sha);
  assert.equal(result.baseline.release.endpoint, "repos/jonathanong/no-mistakes/releases/latest");
  assert.equal(result.compiler, compiler);
  assert.equal(result.harnessCommit, harness);
});

test("allows latest stable release and main to legitimately share an immutable source", () => {
  const result = resolveRefs(
    (endpoint) =>
      endpoint.endsWith("commits/v0.94.0")
        ? responses[Object.keys(responses)[0]]
        : responses[endpoint],
    compiler,
    repository,
    harness,
  );
  assert.equal(result.baseline.commit, result.candidate.commit);
  assert.equal(result.baseline.kind, "release");
});

test("rejects prereleases, drafts, malformed tags, non-GitHub releases, mutable identities and compiler aliases", () => {
  for (const patch of [
    { prerelease: true },
    { draft: true },
    { tag_name: "main" },
    { tag_name: "v1.2.3-rc.1" },
    { id: 0 },
    { html_url: "https://example.com/release" },
  ]) {
    assert.throws(
      () =>
        resolveRefs(
          (endpoint) =>
            endpoint.endsWith("releases/latest")
              ? { ...responses[endpoint], ...patch }
              : responses[endpoint],
          compiler,
          repository,
          harness,
        ),
      /latest published stable release/,
    );
  }
  assert.throws(() => commitIdentity({ sha: "main" }), /immutable commit and tree/);
  assert.throws(() => validateCompiler("stable"), /exact stable/);
  assert.throws(() => validateCompiler("1.93.0-nightly"), /exact stable/);
  assert.throws(
    () => resolveRefs(() => {}, compiler, "bad repository", harness),
    /Invalid repository/,
  );
});

test("keeps identical official release flags, default Mermaid, and an external nonincremental target", () => {
  const manifest = readFileSync(join(fixture, "source/crates/no-mistakes/Cargo.toml"), "utf8");
  assert.match(productionManifest(manifest), /crate-type = \["rlib", "cdylib"\]/);
  assert.throws(
    () => productionManifest(manifest.replace('"mermaid-validation"', '"other"')),
    /default Mermaid/,
  );
  assert.equal(ARGS.includes("--no-default-features"), false);
  verifyEnvironment({ ...FLAGS, RUSTUP_TOOLCHAIN: compiler }, compiler, `${rustc}\n`);
  assert.throws(
    () =>
      verifyEnvironment(
        { ...FLAGS, CARGO_INCREMENTAL: "1", RUSTUP_TOOLCHAIN: compiler },
        compiler,
        `${rustc}\n`,
      ),
    /CARGO_INCREMENTAL/,
  );
  assert.throws(
    () => verifyEnvironment({ ...FLAGS, RUSTUP_TOOLCHAIN: "stable" }, compiler, `${rustc}\n`),
    /Compiler differs/,
  );
  assert.throws(() => externalTarget("/tmp/source", "/tmp/source/target"), /outside/);
  assert.throws(() => externalTarget("/tmp/source", "/tmp/source/..generated"), /outside/);
  assert.equal(externalTarget("/tmp/source", "/tmp/separate-target"), "/tmp/separate-target");
  for (const name of ["no-mistakes", "libno_mistakes-stub.rlib", "no_mistakes-old.d"])
    assert.ok(ownedArtifact("deps", name));
  assert.ok(!ownedArtifact("deps", "libnum_cpus-saved.rlib"));
});

test("restored dependencies cannot ship a stub: remove only owned outputs, rebuild and attest real sources", () =>
  materialize((directory) => {
    dependencyPhase(directory);
    const commands = [];
    build(
      "production",
      join(directory, "source"),
      join(directory, "output"),
      refs,
      "candidate",
      runner(directory, "production", commands),
      environment(directory),
    );
    const provenance = JSON.parse(readFileSync(join(directory, "output/provenance.json"), "utf8"));
    assert.deepEqual(provenance.refs, refs);
    assert.equal(provenance.commit, refs.candidate.commit);
    assert.equal(provenance.tree, refs.candidate.tree);
    assert.equal(provenance.compiler, compiler);
    assert.equal(provenance.removedStubArtifacts, 3);
    assert.equal(
      provenance.sourceHashes["lib.rs"],
      hashFile(join(fixture, "source/crates/no-mistakes/src/lib.rs")),
    );
    assert.equal(
      provenance.sourceManifestSha256,
      hashFile(join(fixture, "source/crates/no-mistakes/Cargo.toml")),
    );
    assert.equal(
      provenance.binarySha256,
      hashFile(join(fixture, "production-target", TARGET, "release/no-mistakes")),
    );
    assert.equal(provenance.cpuProbeSha256, hashFile(join(fixture, "cpu-probe")));
    assert.deepEqual(
      commands.filter(([name, first]) => name === "cargo" && first === "build"),
      [["cargo", ...ARGS]],
    );
  }));

test("rejects stale dependency archives and source identities before a production cargo invocation", () =>
  materialize((directory) => {
    dependencyPhase(directory);
    cpSync(
      join(fixture, "stale-dependency-identity.json"),
      join(directory, "target/hosted-dependency-identity.json"),
    );
    assert.throws(
      () =>
        build(
          "production",
          join(directory, "source"),
          join(directory, "output"),
          refs,
          "candidate",
          runner(directory, "production"),
          environment(directory),
        ),
      /Restored dependencies differ/,
    );
    assert.throws(
      () =>
        build(
          "production",
          join(directory, "source"),
          join(directory, "output"),
          refs,
          "baseline",
          runner(directory, "production"),
          environment(directory),
        ),
      /Checkout differs/,
    );
  }));

test("rejects dependency stubs and unexpected tracked source changes instead of publishing them", () =>
  materialize((directory) => {
    const source = join(directory, "source");
    build(
      "dependencies",
      source,
      join(directory, "output"),
      refs,
      "candidate",
      runner(directory, "dependencies"),
      environment(directory),
    );
    assert.throws(
      () =>
        build(
          "production",
          source,
          join(directory, "output"),
          refs,
          "candidate",
          runner(directory, "production"),
          environment(directory),
        ),
      /Dependency stub cannot/,
    );
    cpSync(join(fixture, "source"), source, { recursive: true });
    const realRunner = runner(directory, "production");
    assert.throws(
      () =>
        build(
          "production",
          source,
          join(directory, "output"),
          refs,
          "candidate",
          (executable, args) =>
            executable === "git" && args[1] === "--name-only"
              ? "crates/no-mistakes/src/lib.rs"
              : realRunner(executable, args),
          environment(directory),
        ),
      /changed sources beyond/,
    );
  }));

test("nightly and demand runs always source-build latest release against resolved main, without a PR perf job", () => {
  const workflow = readFileSync(
    join(__dirname, "../../.github/workflows/hosted-check.yml"),
    "utf8",
  );
  assert.match(workflow, /cron: "23 8 \* \* \*"/);
  assert.match(workflow, /workflow_dispatch:/);
  assert.doesNotMatch(workflow, /pull_request|candidateRef|baselineSourceCommit|0\.86\.2/);
  assert.match(workflow, /cancel-in-progress: false/);
  const jobs = [
    ...workflow.matchAll(/^  ([a-z]+):\n([\s\S]*?)(?=^  [a-z]+:\n|$(?![\s\S]))/gm),
  ].filter(([, , body]) => body.includes("runs-on:"));
  assert.deepEqual(
    jobs.map(([, name]) => name),
    ["resolve", "dependencies", "production", "sample", "summarize"],
  );
  for (const [, name, body] of jobs) {
    const limit = Number(body.match(/^    timeout-minutes: (\d+)$/m)?.[1]);
    assert.ok(limit > 0 && limit <= 30, `${name} must have a concrete job deadline <=30 minutes`);
    for (const [, timeout] of body.matchAll(/^        timeout-minutes: (\d+)$/gm))
      assert.ok(Number(timeout) <= limit);
  }
  for (const phase of ["dependencies", "production"]) {
    const body = jobs.find(([, name]) => name === phase)[2];
    assert.match(
      body,
      /variant: baseline\n            commit: \$\{\{ needs.resolve.outputs.baseline \}\}/,
    );
    assert.match(
      body,
      /variant: candidate\n            commit: \$\{\{ needs.resolve.outputs.candidate \}\}/,
    );
    assert.match(body, /ref: \$\{\{ matrix.commit \}\}/);
    assert.match(
      body,
      /CARGO_TARGET_DIR: \$\{\{ github.workspace \}\}\/\.\.\/no-mistakes-hosted-target/,
    );
    assert.match(body, /RUSTUP_TOOLCHAIN: \$\{\{ needs.resolve.outputs.compiler \}\}/);
  }
  assert.match(workflow, /sample: \[1, 2, 3, 4, 5, 6\]/);
  assert.match(workflow, /jobs: \["1", "4", "auto"\]/);
  assert.equal((workflow.match(/tree: [a-f0-9]{40}/g) || []).length, 2);
  assert.match(workflow, /measure fixture artifact measurements/);
  for (const [, ref] of workflow.matchAll(/uses: ([^\s#]+)/g)) assert.match(ref, /@[a-f0-9]{40}$/);
});
