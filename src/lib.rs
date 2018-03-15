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

struct ColorSetCommand {}

// Implement a redis command to set and get color data.
// Colors can be SET using RGBA hex notation, e.g. cl.COLOR SET pink #ff55efff where the last two bytes are the alpha (will be set to ff if omitted).
// Read colors back with cl.COLOR GET pink
// Colors are stored as bitfields for efficiency.
// -----
// Use WrongArity
//     if argc < 4 {
//        return ffi::RedisModule_WrongArity.unwrap()(ctx);
//    }

impl Command for ColorSetCommand {
    fn name(&self) -> &'static str { "color.set" }
    fn run(&self, r:redis::Redis, args: &[&str]) -> Result<(), ColorError> {
        if args.len() != 3 {
            return Err(error!("Usage: {} COLOR.SET pink #fe55fe", self.name() ));
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
    Command::harness(&ColorSetCommand{}, ctx, argv, argc)
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

    let mut type_functions = raw::RedisModuleTypeMethods {
        version: 1,
        rdb_load: None,
        rdb_save: None,
        aof_rewrite: None,
        free: None,
        mem_usage: None,
        digest: None,
    };

    let type_name = format!("{}\0", "color");
    if raw::create_type(ctx, type_name.as_ptr(), 0, &mut type_functions) == raw::Status::Err {
        return raw::Status::Err
    }

    let command = ColorSetCommand{};
    if raw::create_command(
        ctx, 
        format!("{}\0", command.name()).as_ptr(), 
        Some(Color_RedisCommand), 
        format!("{}\0", command.str_flags() ).as_ptr() ,0,0,0) == raw::Status::Err {
        return raw::Status::Err;
    }
    raw::Status::Ok
}
