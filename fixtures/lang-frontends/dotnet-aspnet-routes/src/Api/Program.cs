namespace Company.Api;

public class Program
{
    public static void Main()
    {
        var app = new WebApplication();
        app.MapGet("/users", UserHandlers.ListUsers);
        app.MapPost("/users", UserHandlers.CreateUser);
        // Unresolved handlers are non-edges.
        app.MapGet("/missing", UnknownHandler);
    }
}

public class WebApplication
{
    public WebApplication MapGet(string path, object handler) => this;
    public WebApplication MapPost(string path, object handler) => this;
}
