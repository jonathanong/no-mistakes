using Xunit;

namespace App.Tests;

public class ValueTests
{
    [Fact]
    public void Value_is_stable()
    {
        Assert.Equal(42, Value.Number);
    }
}
