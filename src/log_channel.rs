pub struct LogChannel {
    file: File,
}

impl LogChannel {
    pub fn create_with_log(path: impl AsRef<Path>) -> (Channel, LogFile) {}
}
