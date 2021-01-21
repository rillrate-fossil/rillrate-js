use napi::{CallContext, Error, JsObject, JsUndefined, Result};
use napi_derive::{js_function, module_exports};
use once_cell::sync::OnceCell;
use rillrate::RillRate;

static RILLRATE: OnceCell<RillRate> = OnceCell::new();

fn js_err(reason: impl ToString) -> Error {
    Error::from_reason(reason.to_string())
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("install", install)?;
    Ok(())
}

#[js_function]
fn install(ctx: CallContext) -> Result<JsUndefined> {
    let rillrate = RillRate::from_env("js").map_err(js_err)?;
    RILLRATE
        .set(rillrate)
        .map_err(|_| js_err("can't install RillRate shared object"))?;
    ctx.env.get_undefined()
}
