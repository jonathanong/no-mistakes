// Dynamic route `/user/:idOrUsername` behind the `(user)` route group. Navigation paths
// whose final segment is an unresolved interpolation must select this page (see #391).
export default function Page() {
  return <main>User</main>;
}
