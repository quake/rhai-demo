// Import from `core` instead of from `std` since we are in no-std mode
use core::{result::Result, str};

// Import heap related library from `alloc`
// https://doc.rust-lang.org/alloc/index.html
use alloc::borrow::ToOwned;

// Import CKB syscalls and structures
// https://docs.rs/ckb-std/
use crate::error::Error;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    debug,
    high_level::{load_cell_capacity, load_script, load_witness_args},
};
use rhai::{Engine, Scope};

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    debug!("script args is {:?}", args);
    if args.is_empty() {
        return Err(Error::MyError);
    }
    let account_chars = str::from_utf8(&args).map_err(|_| Error::Encoding)?;
    debug!("account chars is {:?}", account_chars);

    let witness: Bytes = {
        match load_witness_args(0, Source::Output)?
            .lock()
            .to_opt()
            .map(|bytes| bytes.unpack())
        {
            Some(witness) => witness,
            None => return Err(Error::MyError),
        }
    };

    let customized_rule = str::from_utf8(&witness).map_err(|_| Error::Encoding)?;
    debug!("customized rule is {:?}", customized_rule);

    let price = calc_price(account_chars, customized_rule)?;
    debug!("price is {:?}", price);

    let capacity = load_cell_capacity(0, Source::Output)?;
    if capacity >= (price as u64) * 100_000_000 {
        Ok(())
    } else {
        Err(Error::MyError)
    }
}

pub fn calc_price(account_chars: &str, customized_rule: &str) -> Result<i64, Error> {
    let mut scope = Scope::new();
    scope.push("account_chars", account_chars.to_owned());

    let engine = Engine::new();
    let ast = engine.compile(customized_rule).map_err(|err| {
        debug!("compile error: {:?}", err);
        Error::CustomziedRuleError
    })?;

    engine.eval_ast_with_scope(&mut scope, &ast).map_err(|err| {
        debug!("eval error: {:?}", err);
        Error::CustomziedRuleError
    })
}
