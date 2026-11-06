fn import_call_is_active(file: &TsFileFacts, owner: Option<CallableId>, offset: u32) -> bool {
    if owner.is_some() {
        return true;
    }
    let ranges = &file.import_inactive_call_ranges;
    let after = ranges.partition_point(|(start, _)| *start <= offset);
    after == 0 || offset >= ranges[after - 1].1
}
