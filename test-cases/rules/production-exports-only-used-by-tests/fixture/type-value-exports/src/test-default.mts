// Only tests reference this default: one at runtime, one as a type. The type
// reference must not turn the test-only finding into an advisory.
export default function testOnlyDefault() { return 2; }
