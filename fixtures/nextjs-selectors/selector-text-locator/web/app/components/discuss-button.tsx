import "./cycle";

export function DiscussButton() {
  return (
    <form>
      <button data-pw="discuss-in-community-button">Discuss</button>
      <label htmlFor="email-input">Email address</label>
      <input id="email-input" data-pw="email-input" />
      <button data-pw="email-button">Email</button>
      <button id="save-button"><span>Save changes</span></button>
      <input data-pw="search-input" placeholder="Search" />
      <input data-pw="submit-input" type="submit" value="Send request" />
    </form>
  );
}
