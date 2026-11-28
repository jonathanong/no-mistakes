fn publish<'a>(topic: &str) {
    // FutureRecord::to("ghost")
    let _ = FutureRecord::to("mail.welcome");
    let _ = BaseRecord::to("mail.welcome");
    let _ = FutureRecord::to(topic);
    let _ = FutureRecord::to("mail." + topic);
}

struct FutureRecord;
struct BaseRecord;

impl FutureRecord {
    fn to(_topic: &str) -> Self {
        Self
    }
}

impl BaseRecord {
    fn to(_topic: &str) -> Self {
        Self
    }
}
