// The assignment is conditional.
if (process.env.CI) {
  module.exports = { ci: true };
}
