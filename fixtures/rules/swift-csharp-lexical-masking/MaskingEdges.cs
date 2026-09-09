class MaskingEdges
{
    char ascii = 'a';
    char escaped = '\\u0061';
    char unicode = '\é';
    char widerUnicode = '\𝄞';
    string verbatim = @"literal "" quote";
    string dollarVerbatim = @$"literal {{ value }}";
    string raw = $$$"""literal {{{{ value }}}}""";
}
