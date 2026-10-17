const { primary } = require('@vendor/valkey');
let box = { client: primary };
// Sloppy Annex B setup is eager; target/default/body execution remains repeated.
for (var key = (box.client = require('./ordinary-annex-eager.cjs')) in { a: 1 }) {
  box.client ||= primary;
  require('./ordinary-annex-body.cjs');
}
