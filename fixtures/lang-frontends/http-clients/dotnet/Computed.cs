public static class Computed {
    public static void Load(HttpClient client, string id) {
        client.GetAsync($"/users/{id}");
    }
}
