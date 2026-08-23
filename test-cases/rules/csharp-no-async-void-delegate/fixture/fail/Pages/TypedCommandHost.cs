using System.Threading.Tasks;
using Microsoft.Maui.Controls;

namespace App.Pages;

public sealed class TypedCommandHost
{
    public TypedCommandHost()
    {
        Command = new Command<string>(async value => await UseAsync(value));
    }

    public Command<string> Command { get; }

    private static Task UseAsync(string value) => Task.CompletedTask;
}
