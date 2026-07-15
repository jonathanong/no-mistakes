import "external-package";

// Duplicates report-root/src/helper.ts so unique-exports catches this file if
// supplemental import-usage facts ever leak into the check file universe.
export const helper = false;
