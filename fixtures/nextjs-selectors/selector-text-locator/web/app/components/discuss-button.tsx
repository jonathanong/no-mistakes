export function DiscussButton() {
  return (
    <form>
      <button data-pw="discuss-in-community-button">Discuss</button>
      <label data-pw="email-label">Email</label>
      <button data-pw="email-button">Email</button>
      <button id="save-button">Save</button>
      <input data-pw="search-input" placeholder="Search" />
    </form>
  );
}
