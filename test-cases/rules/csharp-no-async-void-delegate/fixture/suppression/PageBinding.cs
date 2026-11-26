using System.Threading.Tasks;
using Microsoft.Maui.Controls;

public sealed class PageBinding
{
    public Command Inline => new Command(async () => await RefreshAsync()); // no-mistakes-disable-line csharp-no-async-void-delegate: fixture exception

    // no-mistakes-disable-next-line csharp-no-async-void-delegate: fixture exception
    public Command Following => new Command(async () => await RefreshAsync());

    public Command Control => new Command(async () => await RefreshAsync());

    private static Task RefreshAsync() => Task.CompletedTask;
}
