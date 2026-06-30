using Company.App;
using Xunit;

namespace Company.App.Tests;

public class FeedServiceTests
{
    [Fact]
    public void load_returns_feed_title()
    {
        const char quote = '"';
        var path = @"C:\feeds\""rss";
        var service = new FeedService();
        Assert.Equal("feed", service.Load());
        Assert.Equal('"', quote);
        Assert.Contains("feeds", path);
    }
}
