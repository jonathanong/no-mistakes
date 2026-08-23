using System.Threading.Tasks;
using Microsoft.Maui.ApplicationModel;
using Microsoft.Maui.Controls;

namespace App.Pages;

public sealed class LoadablePageBinding
{
    public LoadablePageBinding()
    {
        // new Command(async () => await RefreshAsync());
        RefreshCommand = new Command(() => _ = RefreshAsync());
        MainThread.BeginInvokeOnMainThread(() => _ = RefreshAsync());
    }

    public Command RefreshCommand { get; }

    protected async void OnAppearing()
    {
        await Task.CompletedTask;
    }

    public static Task RunAsync() => Task.Run(async () => await Task.CompletedTask);

    private static Task RefreshAsync() => Task.CompletedTask;
}
