using System.Threading.Tasks;
using Microsoft.Maui.Controls;

namespace App.Pages;

public sealed class MauiCommandHost
{
    public MauiCommandHost()
    {
        Command = new Microsoft.Maui.Controls.Command(async () => await Task.CompletedTask);
    }

    public object Command { get; }
}
