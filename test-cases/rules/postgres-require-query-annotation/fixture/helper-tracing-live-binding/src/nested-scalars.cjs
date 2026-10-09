// Nested conditional scalar values cannot retain SQL proof at a binding join.
const scalar = flag ? (other ? '/* one */ SELECT 1' : 'SELECT 2') : 'SELECT 3';
// Callback alternatives must remain available for opaque capture traversal.
const callback = flag ? (other ? (() => 'SELECT 1') : (() => 'SELECT 2')) : 'SELECT 3';
