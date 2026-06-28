export function SearchLink() {
  return (
    <>
      <a href="/api/v1/search">Plain</a>
      <a href="/api/v1/search?term=widgets&page=2">Matched</a>
      <a href="/api/v1/search?term=widgets&page=2&unused=1">Search</a>
      <a href="/api/v1/search?term=widgets&page=2&sort=name">Sorted</a>
      <a href={`/api/v1/search?${params}`}>Dynamic</a>
    </>
  );
}
