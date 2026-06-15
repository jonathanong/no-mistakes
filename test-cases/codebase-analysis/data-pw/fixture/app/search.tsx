export function Search({ dyn }: { dyn: string }) {
  return (
    <div data-pw="search-bar">
      {/* near miss: a longer attribute name must not match data-pw */}
      <input data-pwx="search-bar" />
      {/* dynamic value must be skipped */}
      <button data-pw={dyn}>go</button>
    </div>
  );
}
