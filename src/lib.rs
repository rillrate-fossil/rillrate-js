use napi::{CallContext, Env, Error, JsObject, JsString, JsUndefined, Result};
use napi_derive::{js_function, module_exports};
use once_cell::sync::OnceCell;
use rillrate::rill::providers::GaugeProvider;
use rillrate::RillRate;

static RILLRATE: OnceCell<RillRate> = OnceCell::new();

fn js_err(reason: impl ToString) -> Error {
    Error::from_reason(reason.to_string())
}

#[js_function]
fn install(ctx: CallContext) -> Result<JsUndefined> {
    let rillrate = RillRate::from_env("js").map_err(js_err)?;
    RILLRATE
        .set(rillrate)
        .map_err(|_| js_err("can't install RillRate shared object"))?;
    ctx.env.get_undefined()
}

#[js_function(1)]
fn gauge_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
    let mut this: JsObject = ctx.this_unchecked();
    let path = arg0.parse().map_err(js_err)?;
    let instance = GaugeProvider::new(path);
    ctx.env.wrap(&mut this, instance)?;
    ctx.env.get_undefined()
}

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    exports.create_named_method("install", install)?;

    let gauge_class = env.define_class(
        "Gauge",
        gauge_constructor,
        &[
      //Property::new(&env, "addCount")?.with_method(add_count),
      //Property::new(&env, "addNativeCount")?.with_method(add_native_count),
    ],
    )?;
    exports.set_named_property("Gauge", gauge_class)?;

    Ok(())
}
