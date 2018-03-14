pub mod raw;
use error::ColorError;
use libc::{c_int, c_long, c_longlong, size_t};
use std::error::Error;
use std::iter;
use std::ptr;
use std::string;

/// `LogLevel` is a level of logging to be specified with a Redis log directive.
#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Debug,
    Notice,
    Verbose,
    Warning,
}

/// Reply represents the various types of a replies that we can receive after
/// executing a Redis command.
#[derive(Debug)]
pub enum Reply {
    Array,
    Error,
    Integer(i64),
    Nil,
    String(String),
    Unknown,
}
/// Command is a basic trait for a new command to be registered with a Redis
/// module.
pub trait Command {
    // Should return the name of the command to be registered.
    fn name(&self) -> &'static str;

    // Run the command.
    fn run(&self, r: Redis, args: &[&str]) -> Result<(), ColorError>;

    // Should return any flags to be registered with the name as a string
    // separated list. See the Redis module API documentation for a complete
    // list of the ones that are available.
    fn str_flags(&self) -> &'static str;
}

impl Command {
    /// Provides a basic wrapper for a command's implementation that parses
    /// arguments to Rust data types and handles the OK/ERR reply back to Redis.
    pub fn harness(
        command: &Command,
        ctx: *mut raw::RedisModuleCtx,
        argv: *mut *mut raw::RedisModuleString,
        argc: c_int,
    ) -> raw::Status {
        let r = Redis { ctx };
        let args = parse_args(argv, argc).unwrap();
        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        match command.run(r, str_args.as_slice()) {
            Ok(_) => raw::Status::Ok,
            Err(e) => {
                raw::reply_with_error(
                    ctx,
                    format!("Cell error: {}\0", e.description()).as_ptr(),
                );
                raw::Status::Err
            }
        }
    }
}

/// Redis is a structure that's designed to give us a high-level interface to
/// the Redis module API by abstracting away the raw C FFI calls.
pub struct Redis {
    ctx: *mut raw::RedisModuleCtx,
}

impl Redis {
  pub fn reply_integer(&self, integer: i64) -> Result<(), ColorError> {
    handle_status(raw::reply_with_long_long(self.ctx, integer as c_longlong), "Could not reply with longlong")
  }
}

fn handle_status(status: raw::Status, message: &str) -> Result<(), ColorError> {
    match status {
        raw::Status::Ok => Ok(()),
        raw::Status::Err => Err(error!(message)),
    }
}

fn parse_args(argv: *mut *mut raw::RedisModuleString, argc: c_int) -> Result<Vec<String>, string::FromUtf8Error> {
  let mut args: Vec<String> = Vec::with_capacity(argc as usize);
  for i in 0..argc {
    let redis_str = unsafe { *argv.offset(i as isize) };
    args.push(manifest_redis_string(redis_str)?);
  }
  Ok(args)
}

fn manifest_redis_string(redis_str: *mut raw::RedisModuleString) -> Result<String, string::FromUtf8Error> {
  let mut length: size_t = 0;
  let bytes = raw::string_ptr_len(redis_str, &mut length);
  from_byte_string(bytes, length)
}

fn from_byte_string(byte_str: *const u8, length: size_t) -> Result<String, string::FromUtf8Error> {
  let mut vec_str: Vec<u8> = Vec::with_capacity(length as usize);
  for j in 0..length {
    let byte: u8 = unsafe { *byte_str.offset(j as isize) };
    vec_str.insert(j, byte);
  }
  String::from_utf8(vec_str)
}