#[macro_use]
extern crate bitflags;
extern crate libc;

#[macro_use]
mod macros;

pub mod error;
mod redis;

use error::ColorError;
use libc::c_int;
use redis::Command;
use redis::raw;

const MODULE_NAME: &str = "redis-color";
const MODULE_VERSION: c_int = 1;

struct ColorCommand {}

impl Command for ColorCommand {
    fn name(&self) -> &'static str { "col.color" }
    fn run(&self, r:redis::Redis, args: &[&str]) -> Result<(), ColorError> {
        if args.len() != 3 && args.len() != 4 {
            return Err(error!("Usage: {} col.COLOR SET pink #fe55fe", self.name() ));
        }
        let key = args[1];
        // TODO what do we want to do here?
        r.reply_integer(99);
        Ok(())
    }
    fn str_flags(&self) -> &'static str { "write" }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn Color_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    Command::harness(&ColorCommand{}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    if raw::init(ctx, format!("{}\0", MODULE_NAME).as_ptr(), MODULE_VERSION, raw::REDISMODULE_APIVER_1 ) == raw::Status::Err {
        return raw::Status::Err;
    }
    let command = ColorCommand{};
    if raw::create_command(
        ctx, 
        format!("{}\0", command.name()).as_ptr(), 
        Some(Color_RedisCommand), 
        format!("{}\0", command.str_flags() ).as_ptr() ,0,0,0) == raw::Status::Err {
        return raw::Status::Err;
    }
    raw::Status::Ok
}
