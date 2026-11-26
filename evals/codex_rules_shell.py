"""Extract static arguments and redirections from one transcript command stage."""
import re
import shlex

_LEXEME = re.compile(
    r"""(?P<operator>[0-9]*(?:<<<|<<|<&|<>|<|>>|>&|>\||>)|&>>|&>)"""
    r"""|(?P<word>(?:'[^']*'|"(?:\\.|[^"\\])*"|\\.|[^\s'"\\<>&]|&(?!>))+)""",
    re.DOTALL,
)
_ASSIGNMENT = re.compile(r"[a-zA-Z_][a-zA-Z_0-9]*=")


def _lexemes(text: str):
    end = 0
    for match in _LEXEME.finditer(text):
        if text[end:match.start()].strip():
            raise ValueError("unrecognized shell text")
        end = match.end()
        operator = match["operator"]
        if operator:
            yield operator.lstrip("0123456789"), "", False
        else:
            word = match["word"]
            yield None, shlex.split(word)[0], bool(_ASSIGNMENT.match(word))
    if text[end:].strip():
        raise ValueError("unrecognized shell text")


def command_arguments(text: str) -> tuple[list[str], list[tuple[str, str]]]:
    argv = []
    redirects = []
    words = iter(_lexemes(text))
    for operator, word, assignment in words:
        if operator:
            target = next(words, None)
            if target is None or target[0] is not None:
                raise ValueError("redirection is missing an operand")
            redirects.append((operator, target[1]))
        elif argv or not assignment:
            argv.append(word)
    return argv, redirects
