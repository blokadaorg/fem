use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::str::FromStr;

use chrono::Utc;
use env_logger;
use env_logger::fmt::Formatter;
use log;

pub fn logger(config: &str) {
  let mut builder = env_logger::Builder::new();

  let log_formatter = plain_formatter;

  builder.format(log_formatter);
  builder.parse_filters(config);
  builder.target(env_logger::Target::Stdout);
  builder.init();
}

fn plain_formatter(fmt: &mut Formatter, record: &log::Record) -> io::Result<()> {
  format(fmt, "ENGINE", record)
}

fn format<W>(fmt: &mut W, tag: &str, record: &log::Record) -> io::Result<()>
where
  W: Write,
{
  let now = Utc::now().format("%H:%M:%S%.3f");
  let level = match record.level() {
    log::Level::Debug => " ",
    log::Level::Error => "E",
    log::Level::Info => " ",
    log::Level::Trace => " ",
    log::Level::Warn => "W",
  };
  let module = record.module_path().unwrap_or("None");
  let line = record.line().unwrap_or(0);
  let args = record.args();
  writeln!(
    fmt,
    "{} {} {:<10} {}:{} {}",
    now, level, tag, module, line, args
  )
}

pub struct FileLogger {
  log_file: File,
  max_level: log::Level,
}

impl log::Log for FileLogger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= self.max_level
  }

  fn log(&self, record: &log::Record) {
    let log_file: &mut File = &mut self.log_file.try_clone().unwrap();
    if self.enabled(record.metadata()) {
      if let Err(e) = format(log_file, "ENGINE", record) {
        println!("{}", e);
      }
    }
  }

  fn flush(&self) {}
}

pub fn file_logger(level: &str, log_file: File) -> Result<FileLogger, log::ParseLevelError> {
  let max_level = log::Level::from_str(level)?;
  Ok(FileLogger {
    max_level,
    log_file,
  })
}

#[cfg(test)]
mod test {
  use super::*;
  use log;
  use log::debug;
  use log::Log;

  #[test]
  fn logging() {
    logger("debug");
    debug!("logging format test");
  }

  #[test]
  fn file_logging() {
    let mut file = File::create("foo.txt").unwrap();
    let l = file_logger("debug", file).unwrap();
    let record = log::Record::builder()
      .args(format_args!("Error!"))
      .level(log::Level::Error)
      .target("myApp")
      .file(Some("server.rs"))
      .line(Some(144))
      .module_path(Some("server"))
      .build();
    l.log(&record);
  }
}
