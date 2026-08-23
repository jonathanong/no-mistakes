namespace App.Pages;

public sealed class CustomHost
{
    public CustomHost()
    {
        Command = new RelayCommand(async () => { });
        DispatchAsync(async () => { });
    }

    public object Command { get; }

    private static void DispatchAsync(System.Action action) { }
}
