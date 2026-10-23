import { IMPORTED_ID } from './ids';

export function DerivedB() {
  return <aside id={IMPORTED_ID}>Imported const owner</aside>; // EXPECT_LOW_UNANALYZABLE_IMPORTED
}
