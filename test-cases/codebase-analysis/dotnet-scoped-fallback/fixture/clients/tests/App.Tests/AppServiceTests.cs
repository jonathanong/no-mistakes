using Company.App;
using Xunit;

namespace Company.App.Tests;

public sealed class AppServiceTests
{
    [Fact]
    public void ReadsName()
    {
        Assert.Equal("app", new AppService().Name());
    }
}
