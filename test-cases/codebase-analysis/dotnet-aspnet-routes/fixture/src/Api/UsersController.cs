namespace Company.Api;

public class UsersController
{
    [HttpGet("/orders")]
    public object GetOrders() => new object();

    [HttpPost("orders")]
    public object CreateOrder() => new object();
}

public class HttpGetAttribute : System.Attribute
{
    public HttpGetAttribute(string template) { }
}

public class HttpPostAttribute : System.Attribute
{
    public HttpPostAttribute(string template) { }
}
