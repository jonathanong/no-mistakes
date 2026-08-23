@Observable
@MainActor
public final class ProfileViewModel {
    public internal(set) var profile: String?
}

@MainActor(unsafe)
public final class SafeViewModel {
    public internal(set) var value = 0
}

struct SettingsViewModel {
    var value = 0
}

actor SyncViewModel {
    var value = 0
}

extension ProfileViewModel {
    func extra() {}
}

@MainActor class SameLineViewModel {
    var value = 0
}

class NotAModel {
    var value = 0
}
