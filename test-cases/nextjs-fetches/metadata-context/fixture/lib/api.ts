export const getUsers = async () => {
  return fetch('/api/users');
};

export async function getPosts() {
  return fetch('/api/posts');
}
