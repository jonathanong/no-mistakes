using Company.App;
using Xunit;

namespace Company.App.Tests;

public class FeedServiceTests
{
    [Fact]
    public void load_returns_feed_title()
    {
        var service = new FeedService();
        Assert.Equal("feed", service.Load());
    }
}
