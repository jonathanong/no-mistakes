const LOCAL_ID = 'recovered-duplicate';

export function DerivedA() {
  return <section id={LOCAL_ID}>Local const owner</section>; // EXPECT_LOW_UNANALYZABLE_LOCAL
}
