namespace App.Tests;

public class ValueTests
{
    [Fact]
    public void ReadsValue() => Assert.Equal(1, Value.Get());
}
