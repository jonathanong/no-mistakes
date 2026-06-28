export default function Page() {
  return (
    <main>
      <button data-pw="save-button">Save</button>
      <label htmlFor="email-input">Email</label>
      <input id="email-input" data-pw="email-input" />
      <input data-pw="search-input" placeholder="Search" />
      <img data-pw="logo-image" alt="Company logo" />
      <button data-pw="help-button" title="Help">?</button>
      <button>Untracked copy</button>
    </main>
  );
}
