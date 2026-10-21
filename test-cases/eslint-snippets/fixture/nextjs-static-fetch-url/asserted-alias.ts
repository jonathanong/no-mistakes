const request = fetch as typeof fetch;
const id = userId;
request(`/api/${id}`);
