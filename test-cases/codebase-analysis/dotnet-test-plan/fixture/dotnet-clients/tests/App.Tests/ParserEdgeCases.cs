using System.Text;
// using Hidden.LineComment;
/* using Hidden.BlockComment; */

namespace Company.App.Tests;

public sealed class ParserEdgeCases
{
    public void KeepsRealReferences()
    {
        const char quote = '"';
        const char slash = '\\';
        var verbatim = @"C:\feeds\""rss";
        var escaped = "CommentedReference \" in string";
        /* var ignored = new CommentedReference(); */
        var service = new ParserLocalReference();
        _ = (quote, slash, verbatim, escaped, service);
    }
}

public sealed class ParserLocalReference;
