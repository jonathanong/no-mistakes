using Company.App;

namespace Company.App
{
    public sealed class BlockNamespace
    {
        public FeedService Service { get; } = new();
    }
}
