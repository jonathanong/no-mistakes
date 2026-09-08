/* new Command(async () => { });
   /* new Command(async () => { }); */
*/
var url = "https://example.test/new Command(async () => { })";
var escaped = "say \\\"new Command(async () => { })\\\"";
var verbatim = @"new Command(async () => { })";
var raw = """new Command(async () => { })""";
var multi = """
new Command(async () => { })
""";
var interpolated = $"{new Command(async () => { })}";
// no-mistakes-disable-next-line csharp-no-async-void-delegate
var suppressed = new Command(async () => { });
var actual = new Command(async () => { });
// In a raw interpolated string, the inner double-brace group is executable;
// the outer braces are literal text grouped around it.
var escapedInterpolation = $"{{new Command(async () => {{ }})}}";
var rawEscapedInterpolation = $$"""{{{{new Command(async () => {{ }})}}}}""";
// Both brace halves are literal text; the rule must consume each complete escape group.
var escapedClosing = $"literal }} new Command(async () => {{ }})";
var rawLiteralBraces = $$"""literal } new Command(async () => { })""";
