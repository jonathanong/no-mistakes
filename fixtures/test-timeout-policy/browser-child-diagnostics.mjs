import { createHash } from "node:crypto";
import { appendFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { availableParallelism, cpus, totalmem } from "node:os";
import { join } from "node:path";

export async function recordBrowserChildDiagnostics(outcome, options) {
  if (outcome.forcedCleanup || outcome.errors.length) {
    try {
      const chronology = outcome.lifecycle.events
        .map((event) => `${Math.round(event.elapsedMs)}ms:${event.phase}`)
        .join(" ");
      console.warn(`Browser SDK pid=${options.child.pid} ${chronology}`);
    } catch (error) {
      console.warn(`Browser SDK chronology unavailable: ${error.message}`);
    }
  }
  try {
    const { child, directory, source, storybook, require } = options;
    const environment = Object.fromEntries(
      [
        "CI",
        "GITHUB_ACTIONS",
        "AI_AGENT",
        "GOMEMLIMIT",
        "NO_COLOR",
        "FORCE_COLOR",
        "GITHUB_SHA",
        "SDK_CHILD_HEAD_SHA",
        "RUNNER_OS",
        "RUNNER_ARCH",
        "ImageOS",
        "ImageVersion",
      ].map((key) => [key, options.environment[key] ?? process.env[key] ?? null]),
    );
    const versions = {};
    for (const name of ["vitest", "@vitest/browser-playwright", "playwright", "vite"]) {
      try {
        versions[name] = JSON.parse(
          await readFile(require.resolve(`${name}/package.json`), "utf8"),
        ).version;
      } catch (error) {
        versions[name] = { error: error.message };
      }
    }
    const receipt = {
      pid: child.pid,
      closed: outcome.closed,
      drained: outcome.drained,
      code: outcome.result?.code,
      signal: outcome.result?.signal,
      forcedCleanup: outcome.forcedCleanup,
      GOMEMLIMIT: process.env.GOMEMLIMIT,
      storybook: storybook ?? null,
      caseHash: createHash("sha256").update(source).digest("hex"),
      node: process.version,
      sdkVersions: versions,
      environment,
      cpu: { available: availableParallelism(), model: cpus()[0]?.model, totalMemory: totalmem() },
      elapsedMs: outcome.lifecycle.elapsedMs,
      lifecycle: outcome.lifecycle,
      errors: outcome.errors.map((error) =>
        error instanceof Error
          ? { name: error.name, message: error.message }
          : { message: String(error) },
      ),
    };
    let browserLifecycle = "";
    if (outcome.drained) {
      try {
        browserLifecycle = await readFile(join(directory, "lifecycle.jsonl"), "utf8");
      } catch (error) {
        if (error.code !== "ENOENT") throw error;
      }
    }
    if (process.env.SDK_CHILD_RECEIPT_FILE) {
      await appendFile(process.env.SDK_CHILD_RECEIPT_FILE, JSON.stringify(receipt) + "\n");
      const artifacts = process.env.SDK_CHILD_RECEIPT_FILE + "-artifacts/" + child.pid;
      await mkdir(artifacts, { recursive: true });
      await writeFile(
        join(artifacts, "lifecycle.jsonl"),
        outcome.lifecycle.events
          .map((event) => JSON.stringify({ origin: "observer", ...event }))
          .join("\n") +
          "\n" +
          browserLifecycle,
      );
      await writeFile(join(artifacts, "output.log"), outcome.output);
    }
    if ((outcome.forcedCleanup || outcome.errors.length) && browserLifecycle) {
      const events = browserLifecycle.trim().split("\n");
      const chronology = events
        .slice(-16)
        .map((line) => {
          try {
            const event = JSON.parse(line);
            return `${Math.round(event.atMs)}ms:${event.phase}`;
          } catch {
            return "unparseable-event";
          }
        })
        .join(" ");
      console.warn(`Browser SDK child lifecycle (${events.length} events, last 16): ${chronology}`);
    }
  } catch (error) {
    console.warn(`Browser SDK diagnostics unavailable: ${error.message}`);
  }
}
