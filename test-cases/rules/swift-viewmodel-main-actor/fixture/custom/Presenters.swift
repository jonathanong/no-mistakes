@UIActor
final class SafePresenter {
    var value = 0
}

final class BrokenPresenter {
    var value = 0
}

@MainActor
final class IgnoredViewModel {
    var value = 0
}
