extern crate libc;
#[macro_use]
extern crate emacs;
extern crate serde_json;
extern crate time;

use emacs::{Env, CallEnv, Result, Value, IntoLisp};

emacs_plugin_is_GPL_compatible!();
emacs_module_init!(init);

fn json_to_lisp(env: &Env, value: serde_json::Value) -> Result<Value> {

    use serde_json::Value::*;

    match value {
        Null => env.intern("nil"),
        Bool(val) => {
            if val {
                env.intern("t")
            } else {
                env.intern("nil")
            }
        },
        Number(n) => {
            if n.is_i64() {
                n.as_i64().unwrap().into_lisp(env)
            } else if n.is_f64() {
                n.as_f64().unwrap().into_lisp(env)
            } else if n.is_u64() {
                (n.as_u64().unwrap() as i64).into_lisp(env)
            } else {
                env.message(&format!("UNKNOWN NUMBERS: {}", n));
                env.intern("nil")
            }
        },
        String(str) => {
            str.into_lisp(env)
        },
        Array(array) => {
            let mut vec = Vec::with_capacity(array.len());
            for item in array {
                vec.push(json_to_lisp(env, item)?);
            }
            env.call("list", &vec)
        },
        Object(object) => {
            let size = object.len() as i64;
            let hashtable = env.call("make-hash-table", &[
                env.intern(":test")?,
                env.intern("equal")?,
                env.intern(":size")?,
                size.into_lisp(env)?
            ])?;

            for (key, value) in object.iter() {
                env.call("puthash", &[
                    key.into_lisp(env)?,
                    json_to_lisp(env, value.clone())?,
                    hashtable
                ])?;
            }

            Ok(hashtable)
        },
    }
}

// use time::PreciseTime;

fn parse_string(env: &CallEnv) -> Result<Value> {
    let string: String = env.parse_arg(0)?;

    // let start = PreciseTime::now();
    let res = match serde_json::from_str(string.as_str()) {
        Ok(value) => json_to_lisp(env, value),
        Err(_) => env.intern("nil")
    };
    // let end = PreciseTime::now();
    // env.message(&format!("\nPARSE_STRING: {}, {} seconds\n", string.len(), start.to(end)))?;
    res
}

fn init(env: &Env) -> Result<Value> {
    emacs_export_functions! {
        env, "emacs-json-", {
            "parse-string" => (parse_string, 1..1)
        }
    }
    env.provide("emacs-json")
}
