using System.Threading.Tasks;
using Microsoft.Maui.ApplicationModel;

namespace App.Pages;

public sealed class SessionBinding
{
    public SessionBinding()
    {
        MainThread.BeginInvokeOnMainThread(async () => await RefreshAsync());
    }

    private static Task RefreshAsync() => Task.CompletedTask;
}
