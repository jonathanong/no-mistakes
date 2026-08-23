using System.Threading.Tasks;
using Microsoft.Maui.Controls;

namespace App.Pages;

public sealed class LoadablePageBinding
{
    public LoadablePageBinding()
    {
        RefreshCommand = new Command(async () => await RefreshAsync());
    }

    public Command RefreshCommand { get; }

    private static Task RefreshAsync() => Task.CompletedTask;
}
