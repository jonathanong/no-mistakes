using Company.Other;
using Xunit;

namespace Company.Other.Tests;

public sealed class OtherServiceTests
{
    [Fact]
    public void ReadsName()
    {
        Assert.Equal("other", new OtherService().Name());
    }
}
