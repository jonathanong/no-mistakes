import type { Plugin } from "vitest/config";

export interface NodeRunnerPolicy {
  /** Absolute final custom runner module, or null for Vitest's native runner. */
  runner: string | null;
}

/** Route Node registration imports and protect the final native or custom runner. */
export declare function testTimeoutPolicyPlugin(
  maximum?: number,
  registrationPackages?: readonly string[],
  nodeRunner?: NodeRunnerPolicy,
): Plugin;

/** Route the native Vitest Browser Mode runner and registrar dependency closure. */
export declare function browserTestTimeoutPolicyPlugin(
  maximum?: number,
  registrationPackages?: readonly string[],
): Plugin;
