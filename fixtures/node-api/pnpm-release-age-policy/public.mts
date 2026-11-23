import type {
  PnpmReleaseAgePermanentPackage,
  PnpmReleaseAgePolicyOptions,
  PnpmReleaseAgeTemporaryGroup,
} from "no-mistakes";

const exact: PnpmReleaseAgePermanentPackage = { name: "acme-lib" };
const glob: PnpmReleaseAgePermanentPackage = {
  pattern: "acme-cli-*",
  reason: "First-party platform packages",
};
const temporary: PnpmReleaseAgeTemporaryGroup = {
  selectors: ["demo-temporary-package@9.9.9"],
  reason: "Upstream release regression",
  eligibleForRemovalAt: "2027-01-02T03:04:05Z",
};

export const releaseAge: PnpmReleaseAgePolicyOptions = {
  permanentPackages: [exact, glob],
  temporaryGroups: [temporary],
};

// @ts-expect-error A permanent entry requires exactly one of name and pattern.
export const both: PnpmReleaseAgePermanentPackage = { name: "acme-lib", pattern: "*" };
// @ts-expect-error A reason alone does not specify the exempt package.
export const neither: PnpmReleaseAgePermanentPackage = { reason: "first-party" };
