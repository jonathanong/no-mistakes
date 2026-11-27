public static class Client {
    public static void Load(HttpClient client) {
        client.GetAsync("http://localhost:8080/users");
    }
}
