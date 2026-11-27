public static class Routes {
    public static void Map(WebApplication app) {
        app.MapGet("/users", Handlers.ListUsers);
    }
}
