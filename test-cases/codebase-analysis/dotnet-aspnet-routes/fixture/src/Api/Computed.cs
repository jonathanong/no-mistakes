namespace Company.Api;

public static class Computed
{
    // v1 skips computed paths, lambdas, and [HttpGet] with no template.
    public static void Register(WebApplication app, string path)
    {
        app.MapGet(path, UserHandlers.ListUsers);
        app.MapGet("/computed", () => "ok");
    }

    [HttpGet]
    public static object Index() => new object();

    [HttpGet(Name = "named")]
    public static object Named() => new object();
}

public class HttpGetNamedAttribute : System.Attribute
{
    public string Name { get; set; } = "";
}
