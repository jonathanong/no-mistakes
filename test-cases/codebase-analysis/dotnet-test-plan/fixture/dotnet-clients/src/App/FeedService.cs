namespace Company.App;

public class FeedService
{
    private readonly FeedClient client = new();

    public string Load() => client.Title();
}
