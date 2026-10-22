// `true && require` is eager setup reachability at effort max and an
// unanalyzable shape at effort low. Do not rewrite it to a bare require.
require('./require-helper.cjs')
true && require('./conditional-helper.cjs') // EXPECT_SHAPE
void require('./void-helper.cjs') // EXPECT_SHAPE
require('./seq-a.cjs'), // EXPECT_SHAPE
  require('./seq-b.cjs') // EXPECT_SHAPE
