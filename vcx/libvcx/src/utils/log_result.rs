use log::LogLevel;
use std::fmt;

pub trait LogResult {
    type Ok;
    type Error;
    fn error_log(self:Self) -> Result<Self::Ok, Self::Error>;
    fn error_log_with_ctx(self:Self, ctx: &str) -> Result<Self::Ok, Self::Error>;
    fn warn_log(self:Self) -> Result<Self::Ok, Self::Error>;
    fn warn_log_with_ctx(self:Self, ctx: &str) -> Result<Self::Ok, Self::Error>;
    fn log_result(self:Self, level: LogLevel, ctx: Option<&str>) -> Result<Self::Ok, Self::Error>;
}

fn _log_result<T, E: fmt::Debug>(level: LogLevel, ctx: Option<&str>, error: Result<T, E> ) -> Result<T, E> {
    match error {
        Ok(t) => Ok(t),
        Err(e) => {
            match ctx {
                Some(c) => {
                    log!(level, "{} -- {:?}", c, e);
                }
                None => {
                    log!(level, "error with -- {:?}", e);
                }
            }
            Err(e)
        }
    }
}


impl<T,E: fmt::Debug> LogResult for Result<T, E> {
    type Ok = T;
    type Error = E;

    fn error_log(self: Self) -> Result<Self::Ok, Self::Error> {
        _log_result(LogLevel::Error, None, self)
    }

    fn error_log_with_ctx(self: Self, ctx: &str) -> Result<Self::Ok, Self::Error> {
        _log_result(LogLevel::Error, Some(ctx), self)
    }

    fn warn_log(self: Self) -> Result<Self::Ok, Self::Error> {
        _log_result(LogLevel::Warn, None, self)
    }

    fn warn_log_with_ctx(self: Self, ctx: &str) -> Result<Self::Ok, Self::Error> {
        _log_result(LogLevel::Warn, Some(ctx), self)
    }

    fn log_result(self: Self, level: LogLevel, ctx: Option<&str>) -> Result<Self::Ok, Self::Error> {
        _log_result(level, ctx, self)
    }
}


#[cfg(test)]
mod tests {
    extern crate tempfile;
    use super::*;
    use log;
    use log::{LogRecord, LogMetadata, LogLevelFilter};
    use std::path::{Path, PathBuf};
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};

    use self::tempfile::NamedTempFile;


    struct TestLogger {
        path: String,
        temp_file: NamedTempFile
    }

    impl TestLogger {
        fn new() -> TestLogger {
            TestLogger{
                path: String::from("/tmp/temp"),
                temp_file: NamedTempFile::new().unwrap()
            }
        }

    }

    impl log::Log for TestLogger {
        fn enabled(&self, metadata: &LogMetadata) -> bool {
            true
        }

        fn log(&self, record: &LogRecord) {
            if self.enabled(record.metadata()) {
                let msg = format!("{} - {}\n", record.level(), record.args());
                let p = Path::new(&self.path);
                let mut f = OpenOptions::new().append(true).create(true).open(self.temp_file.path()).unwrap();
                f.write_all(msg.as_bytes()).unwrap();
            }
        }
    }

    fn _get_log(path: PathBuf) -> String {
        let mut rtn = String::new();
        let mut f = File::open(path).unwrap();
        f.read_to_string(&mut rtn).unwrap();

        rtn
    }

    #[test]
    fn see_result_in_log_test() {
        let log = TestLogger::new();
        let log_path = log.temp_file.path().to_owned();

        log::set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Trace);
            Box::new(log)
        }).unwrap();

        let test: Result<(), &str> = Err("FINDME");
        test.error_log_with_ctx("THERFORE").unwrap_err();

        let logs = _get_log(log_path);
        assert!(logs.contains("FINDME"));
        assert!(logs.contains("THERFORE"));

    }

}