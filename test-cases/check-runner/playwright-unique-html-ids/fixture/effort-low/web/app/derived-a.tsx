export function DerivedA() {
  const LOCAL_ID = 'recovered-duplicate';
  return <section id={LOCAL_ID}>Local const owner</section>; // EXPECT_LOW_UNANALYZABLE_LOCAL
}
