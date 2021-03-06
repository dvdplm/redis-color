#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
mod macros;

pub mod error;
mod redis;

use std::fmt;
use error::ColorError;
use libc::c_int;
use redis::Command;
use redis::raw;

const MODULE_NAME: &str = "redis-color";
const MODULE_VERSION: c_int = 1;

#[derive(Debug)]
pub struct Color { r: u8, g: u8, b: u8, a: u8 }

impl Color {
    pub fn new() -> Color {
        Color{r:58,g:64,b:12,a:78}
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:x}{:x}{:x}", self.r, self.g, self.b)
    }
}

// Implement a redis command to set and get color data.
// Colors can be SET using RGBA hex notation, e.g. cl.COLOR SET pink #ff55efff where the last two bytes are the alpha (will be set to ff if omitted).
// Read colors back with cl.COLOR GET pink
// Colors are stored as bitfields for efficiency.
// -----
// Use WrongArity
//     if argc < 4 {
//        return ffi::RedisModule_WrongArity.unwrap()(ctx);
//    }
struct SetColorCommand {}
impl Command for SetColorCommand {
    fn name(&self) -> &'static str { "color.set" }
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), ColorError> {
        if args.len() != 3 {
            return Err(error!("Usage: {} COLOR.SET pink #fe55fe", self.name() ));
        }
        let key = r.open_key_writable(args[1]);
        if key.is_empty() {
            log_debug!(r, "Key {:?} is empty. Writing {:?}.", key, args[2]);
            // TODO: parse color and write bytes back
            let c = Color::new();
            let out = key.write(&c)?;
            log_debug!(r, "Wrote value {:?} to key {:?}. Result: {:?}", c, key, out);
        } else {
            // r.log_debug("Key is NOT empty. TODO: check type, bail if not our type, overwrite otherwise");
            // let kt = key.key_type();
            // log_debug!(r, "KEY TYPE: {:?}", kt);
            if !key.valid_key_type() {
                return Err(error!(raw::ERRORMSG_WRONGTYPE)) // TODO: better error msg
            }
        }
        r.reply_simple_string(raw::SIMPLE_OK)?;
        Ok(())
    }
    fn str_flags(&self) -> &'static str { "write" }
}
struct GetColorCommand{}
impl Command for GetColorCommand {
    fn name(&self) -> &'static str { "color.get" }
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), ColorError> {
        // TODO: check arity
        let key = r.open_key(args[1]);
        if !key.valid_key_type() {
            return Err(error!(raw::ERRORMSG_WRONGTYPE))
        }
        let color_res = key.read();
        match color_res {
            Ok(cr) => {
                log_debug!(r, "Read color from key {:?}: {:?}", key, cr);
                let c: &mut Color = unsafe {&mut *(cr as *mut Color)};
                log_debug!(r, "Cast color: {:?}", c);
                r.reply_simple_string(&format!("{}", c))?;
                Ok(())
            },
            Err(e) => {
                Err(error!("No."))
            }
        }
    }
    fn str_flags(&self) -> &'static str { "readonly" }
}
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn SetColor_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int,
) -> raw::Status {
    Command::harness(&SetColorCommand{}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn GetColor_RedisCommand(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: c_int
) -> raw::Status {
    Command::harness(&GetColorCommand{}, ctx, argv, argc)
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

    let type_name = format!("{}\0", "dvd-color");
    if raw::create_type(ctx, type_name.as_ptr(), 0, &mut type_functions) == raw::Status::Err {
        return raw::Status::Err
    }

    let set_cmd = SetColorCommand{};
    if raw::create_command(
        ctx, 
        format!("{}\0", set_cmd.name()).as_ptr(), 
        Some(SetColor_RedisCommand), 
        format!("{}\0", set_cmd.str_flags() ).as_ptr(), 0, 0, 0) == raw::Status::Err {
        return raw::Status::Err;
    }

    let get_cmd = GetColorCommand{};
    if raw::create_command(
        ctx, 
        format!("{}\0", get_cmd.name()).as_ptr(), 
        Some(GetColor_RedisCommand), 
        format!("{}\0", get_cmd.str_flags() ).as_ptr(), 0, 0, 0) == raw::Status::Err {
            return raw::Status::Err;
        }
    raw::Status::Ok
}
