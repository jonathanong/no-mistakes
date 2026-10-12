# SDK project sources

These checked-in sources supply the real Vitest, Browser Mode, and Storybook
projects used by the timeout policy integration tests. The fixture helpers copy
static source or interpolate numbered `{{0}}` settings into the saved templates
before starting a child SDK. Settings are explicitly passed by each helper or
case; the template reader never evaluates source text.

The child project receives a private temporary directory because Vitest writes
runtime receipts and optimizer artifacts. Those runtime outputs and generated
policy facade modules remain actual SDK/product output, not fixture inputs.
