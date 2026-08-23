// no-mistakes-disable-file csharp-no-async-void-delegate
using System.Threading.Tasks;
using Microsoft.Maui.Controls;

namespace App.Pages;

public sealed class Disabled
{
    public Disabled()
    {
        Command = new Command(async () => await Task.CompletedTask);
    }

    public Command Command { get; }
}
