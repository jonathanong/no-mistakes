import { browserChildContainment } from "../../../fixtures/test-timeout-policy/browser-child-containment.mjs";
import { containmentChildProbe } from "../../../fixtures/test-timeout-policy/containment-child-probe.mjs";
import { observeSdkChild } from "../../../fixtures/test-timeout-policy/owned-sdk-child.mjs";
import { describe, expect, it, vi } from "vitest";
import { configDefaults, mergeConfig } from "vitest/config";
import { browserTestTimeoutPolicyPlugin, testTimeoutPolicyPlugin } from "./plugin.mjs";
import { replaceStorybookAnnotationSetup } from "../../../fixtures/test-timeout-policy/storybook-timeout-fixture.mjs";
describe("browser routing policy contracts", () => {
  it("preserves inline-all sentinels in both public plugin configuration routes", () => {
    for (const plugin of [
      testTimeoutPolicyPlugin(30000, ["registrar"]),
      browserTestTimeoutPolicyPlugin(30000, ["registrar"]),
    ]) {
      const config = { test: { server: { deps: { inline: true } } } };
      expect(plugin.config(config).test.server.deps.inline).toBe(true);
      const patterns = [/owned/, "existing"];
      expect(
        plugin.config({ test: { server: { deps: { inline: patterns } } } }).test.server.deps.inline,
      ).toEqual(["registrar"]);
      expect(patterns).toEqual([/owned/, "existing"]);
    }
  });
  it("composes client optimizers without modifying server configuration or caller arrays", () => {
    const plugin = browserTestTimeoutPolicyPlugin(30000, ["registrar"]);
    const include = ["dependency > registrar/subpath", "registrar-extra", "ordinary"];
    const config = { optimizeDeps: { include, exclude: ["ordinary-exclusion", "vitest"] } };
    expect(plugin.configEnvironment.handler("ssr", config)).toBeUndefined();
    for (const name of ["client", "custom"]) {
      const result = plugin.configEnvironment.handler(name, { ...config, consumer: "client" });
      expect(result?.optimizeDeps.include).toEqual(["registrar-extra", "ordinary"]);
      expect(result?.optimizeDeps.exclude).toEqual([
        "ordinary-exclusion",
        "vitest",
        "@vitest/browser",
        "registrar",
      ]);
    }
    expect(plugin.configEnvironment.handler("client", {})?.optimizeDeps.include).toEqual([]);
    expect(include).toEqual(["dependency > registrar/subpath", "registrar-extra", "ordinary"]);
  });
  it("preserves default, single and custom reporter registrations before the browser lifecycle guard", () => {
    const plugin = browserTestTimeoutPolicyPlugin();
    const custom = { onInit() {} };
    expect(plugin.config().test.reporters.slice(0, -1)).toEqual(configDefaults.reporters);
    const single = { test: { reporters: "verbose" } };
    expect(mergeConfig(single, plugin.config(single)).test.reporters[0]).toBe("verbose");
    const reporters = ["json", custom];
    const merged = mergeConfig({ test: { reporters } }, plugin.config({ test: { reporters } }));
    expect(merged.test.reporters.slice(0, 2)).toEqual(reporters);
    expect(merged.test.reporters).toHaveLength(3);
    expect(reporters).toEqual(["json", custom]);
  });
  it("removes late aggregated registrar bundles from every public client configuration before browser startup", () => {
    const plugin = browserTestTimeoutPolicyPlugin(30000, ["registrar"]);
    const reporter = plugin.config().test.reporters.at(-1);
    if (
      !reporter ||
      typeof reporter === "string" ||
      !("onBrowserInit" in reporter) ||
      typeof reporter.onBrowserInit !== "function"
    )
      throw new Error("Browser initialization guard missing");
    const optimizer = () => ({
      include: [
        "dependency > registrar/subpath",
        "vitest > expect-type",
        "registrar-extra",
        "ordinary",
      ],
    });
    const root = optimizer();
    const declared = optimizer();
    const server = optimizer();
    const active = optimizer();
    const project = {
      viteConfig: { optimizeDeps: root, environments: { client: { optimizeDeps: declared } } },
      vite: {
        config: { optimizeDeps: server },
        environments: { client: { config: { optimizeDeps: active } } },
      },
    };
    Reflect.apply(reporter.onBrowserInit, reporter, [project]);
    for (const config of [root, declared, server, active])
      expect(config.include).toEqual(["registrar-extra", "ordinary"]);
    Reflect.apply(reporter.onBrowserInit, reporter, [
      { viteConfig: {}, vite: { config: {}, environments: {} } },
    ]);
    Reflect.apply(reporter.onBrowserInit, reporter, [
      {
        viteConfig: {},
        vite: { config: {}, environments: { client: { config: { optimizeDeps: {} } } } },
      },
    ]);
  });
  it("rejects disabled or incorrectly excluded browser projects before filtering their bundles", () => {
    const plugin = browserTestTimeoutPolicyPlugin();
    expect(() => plugin.configResolved({})).toThrow("enabled browser project");
    expect(() => plugin.configResolved({ test: { browser: { enabled: true } } })).toThrow(
      "Missing browser timeout routing exclusion",
    );
    const config = {
      test: { browser: { enabled: true } },
      optimizeDeps: { exclude: plugin.config().optimizeDeps.exclude },
    };
    expect(() => plugin.configResolved(config)).not.toThrow();
  });
  it("removes exact, subpath and any nested owner token while preserving near-prefix and unrelated order", () => {
    const plugin = browserTestTimeoutPolicyPlugin(30000, ["@storybook/addon-vitest"]);
    const include = [
      "@storybook/addon-vitest",
      "@storybook/addon-vitest/internal/setup-file",
      "dependency > @storybook/addon-vitest/internal/setup-file",
      "@storybook/addon-vitest/internal/setup-file > dependency",
      "dependency > middle > @storybook/addon-vitest",
      " dependency>@storybook/addon-vitest/internal/setup-file > leaf ",
      "vitest/browser",
      "unrelated-first",
      "@storybook/addon-vitest-extra",
      "dependency > @storybook/addon-vitest-extra/internal/setup-file",
      "vitest-extra",
      "unrelated-last",
    ];
    const config = {
      test: { browser: { enabled: true } },
      optimizeDeps: {
        include,
        exclude: plugin.config().optimizeDeps.exclude,
      },
    };
    plugin.configResolved(config);
    expect(config.optimizeDeps.include).toEqual([
      "unrelated-first",
      "@storybook/addon-vitest-extra",
      "dependency > @storybook/addon-vitest-extra/internal/setup-file",
      "vitest-extra",
      "unrelated-last",
    ]);
    expect(include).toHaveLength(12);
    expect(config.optimizeDeps.exclude).toEqual(plugin.config().optimizeDeps.exclude);
  });
  it.each([
    {},
    { test: {} },
    { test: { setupFiles: [] } },
    { test: { setupFiles: ["unrelated.mts"] } },
  ])("rejects absent or nonmatching addon annotations setup", (config) => {
    expect(() => replaceStorybookAnnotationSetup(config, "/stable.mts")).toThrow(
      "annotations setup missing",
    );
  });
  it("replaces string and array addon setup without dropping other setup files", () => {
    const config = {
      test: {
        setupFiles: [
          "before.mts",
          "@storybook/addon-vitest/internal/setup-file-with-project-annotations",
          "after.mts",
        ],
      },
    };
    replaceStorybookAnnotationSetup(config, "/stable.mts");
    expect(config.test.setupFiles).toEqual(["before.mts", "/stable.mts", "after.mts"]);
    const single = {
      test: { setupFiles: "/addon/setup-file-with-project-annotations.js" },
    };
    replaceStorybookAnnotationSetup(single, "/stable.mts");
    expect(single.test.setupFiles).toEqual(["/stable.mts"]);
  });
  it.each([false, true])(
    "bounds SDK startup independently of native deadlines; storybook=%s",
    async (storybook) => {
      vi.useFakeTimers();
      try {
        const child = containmentChildProbe();
        const terminate = vi.fn(() => {
          child.emit("close", null, "SIGKILL");
          return true;
        });
        const containment = browserChildContainment(storybook);
        const observed = observeSdkChild(child, { ...containment, capture: () => terminate });
        await vi.advanceTimersByTimeAsync(4999);
        expect(terminate).not.toHaveBeenCalled();
        if (storybook) {
          await vi.advanceTimersByTimeAsync(15000);
          expect(terminate).not.toHaveBeenCalled();
        }
        await vi.advanceTimersByTimeAsync(1);
        const outcome = await observed;
        expect(terminate).toHaveBeenCalledOnce();
        expect(outcome.result).toEqual({ code: null, signal: "SIGKILL" });
        expect(outcome.forcedCleanup).toBe(true);
        expect(outcome.drained).toBe(true);
        expect(outcome.errors).toEqual([]);
        // Leave room for both close and group-drain barriers under the parent suite.
        expect(containment.watchdogMs + 2000).toBeLessThan(30000);
      } finally {
        vi.useRealTimers();
      }
    },
  );
});
