// Keep this replacement a different byte length from the initial selector.
// The integration test copies it immediately after committing the initial
// file, so an equal-size replacement can hit Git's racy-clean shortcut and
// make the streamed diff omit the selector change.
export const Selector = () => <div data-testid="new-selector-replaced" />;
