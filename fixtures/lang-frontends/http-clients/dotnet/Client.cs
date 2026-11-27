public static class Client {
    public static void Load(HttpClient client) {
        client.GetAsync("/users");
    }
}
