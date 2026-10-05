use super::*;

impl BodyFinder<'_, '_> {
    pub(super) fn collect(&mut self, value: Value) {
        self.collect_bounded(value, &mut 4096, 0);
    }
    fn collect_bounded(&mut self, value: Value, remaining: &mut usize, depth: usize) {
        if *remaining == 0 || depth >= 64 {
            self.incomplete = true;
            return;
        }
        *remaining -= 1;
        match value {
            Value::Array(values) => {
                for value in values.iter().cloned() {
                    self.collect_bounded(value, remaining, depth + 1);
                }
            }
            Value::Object(properties, complete) => {
                let mut properties = (*properties).clone();
                self.incomplete |= !complete;
                if let Some((destination, offset)) = properties.remove("destination") {
                    self.saw_destination_property = true;
                    if let Value::String(value) = destination {
                        let line = byte_offset_to_line(self.source, offset as usize) as usize;
                        self.destinations.entry(value).or_insert(line);
                    } else {
                        self.incomplete = true;
                    }
                } else if self.name == "rewrites" {
                    for key in ["beforeFiles", "afterFiles", "fallback"] {
                        if let Some((value, _)) = properties.remove(key) {
                            self.collect_bounded(value, remaining, depth + 1);
                        }
                    }
                    self.incomplete |= !properties.is_empty();
                } else {
                    self.incomplete = true;
                }
            }
            _ => self.incomplete = true,
        }
    }
}
