// Sibling literal route `/user/settings`. An interpolated navigation must NOT select this
// page — an unresolved value is not assumed to equal the concrete `settings` segment.
export default function Page() {
  return <main>User settings</main>;
}
