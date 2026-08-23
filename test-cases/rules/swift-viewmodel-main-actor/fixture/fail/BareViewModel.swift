class BareViewModel {
    var value = 0
}

) class UnmatchedParenViewModel {
    var value = 0
}

()
class EmptyParenViewModel {
    var value = 0
}

@Foo(bar)
class AttrArgsViewModel {
    var value = 0
}

@Foo(a(b))
class NestedArgsViewModel {
    var value = 0
}

Foo()
class CallArgsViewModel {
    var value = 0
}

{ class BraceViewModel
    var value = 0
}

// class CommentedViewModel {}
