fn publish(topic: &str) {
    let _ = FutureRecord::to("mail.welcome");
    let _ = BaseRecord::to("mail.welcome");
    let _ = FutureRecord::to(topic);
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
