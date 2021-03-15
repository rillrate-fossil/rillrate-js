use napi::{
    CallContext, Env, Error, JsBoolean, JsNumber, JsObject, JsString, JsUndefined, Property, Result,
};
use napi_derive::{js_function, module_exports};
use once_cell::sync::OnceCell;
use rillrate::RillRate;
use rillrate::{Counter, Dict, Gauge, Histogram, Logger, Pulse};
use std::convert::TryInto;

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

macro_rules! js_decl {
    (@new $cls:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
            let mut this: JsObject = ctx.this_unchecked();
            let instance = $cls::create(&arg0).map_err(js_err)?;
            ctx.env.wrap(&mut this, instance)?;
            ctx.env.get_undefined()
        }
    };

    (@bool $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsBoolean> {
            let this: JsObject = ctx.this_unchecked();
            let provider: &mut $cls = ctx.env.unwrap(&this)?;
            ctx.env.get_boolean(provider.$meth())
        }
    };

    (@f64 $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let arg0: f64 = ctx.get::<JsNumber>(0)?.try_into()?;
            let this: JsObject = ctx.this_unchecked();
            let provider: &mut $cls = ctx.env.unwrap(&this)?;
            provider.$meth(arg0);
            ctx.env.get_undefined()
        }
    };

    (@str $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
            let this: JsObject = ctx.this_unchecked();
            let provider: &mut $cls = ctx.env.unwrap(&this)?;
            provider.$meth(arg0);
            ctx.env.get_undefined()
        }
    };

    (@two_str $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(2)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
            let arg1 = ctx.get::<JsString>(1)?.into_utf8()?.into_owned()?;
            let this: JsObject = ctx.this_unchecked();
            let provider: &mut $cls = ctx.env.unwrap(&this)?;
            provider.$meth(arg0, arg1);
            ctx.env.get_undefined()
        }
    };
}

js_decl!(@new Counter, counter_constructor);
js_decl!(@bool Counter, is_active, counter_is_active);
js_decl!(@f64 Counter, inc, counter_inc);

#[js_function(3)]
fn gauge_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
    let arg1: f64 = ctx.get::<JsNumber>(1)?.try_into()?;
    let arg2: f64 = ctx.get::<JsNumber>(2)?.try_into()?;
    let mut this: JsObject = ctx.this_unchecked();
    let instance = Gauge::create(&arg0, arg1, arg2).map_err(js_err)?;
    ctx.env.wrap(&mut this, instance)?;
    ctx.env.get_undefined()
}

js_decl!(@bool Gauge, is_active, gauge_is_active);
js_decl!(@f64 Gauge, set, gauge_set);

js_decl!(@new Pulse, pulse_constructor);
js_decl!(@bool Pulse, is_active, pulse_is_active);
js_decl!(@f64 Pulse, inc, pulse_inc);
js_decl!(@f64 Pulse, dec, pulse_dec);
js_decl!(@f64 Pulse, set, pulse_set);

#[js_function(2)]
fn histogram_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let arg0 = ctx.get::<JsString>(0)?.into_utf8()?.into_owned()?;
    let arg1 = ctx.get::<JsObject>(1)?;
    let len = arg1.get_array_length()?;
    let mut levels = Vec::new();
    for idx in 0..len {
        let value: f64 = arg1.get_element::<JsNumber>(idx)?.try_into()?;
        levels.push(value);
    }
    let mut this: JsObject = ctx.this_unchecked();
    let instance = Histogram::create(&arg0, &levels).map_err(js_err)?;
    ctx.env.wrap(&mut this, instance)?;
    ctx.env.get_undefined()
}

js_decl!(@bool Histogram, is_active, histogram_is_active);
js_decl!(@f64 Histogram, add, histogram_add);

js_decl!(@new Dict, dict_constructor);
js_decl!(@bool Dict, is_active, dict_is_active);
js_decl!(@two_str Dict, set, dict_set);

js_decl!(@new Logger, logger_constructor);
js_decl!(@bool Logger, is_active, logger_is_active);
js_decl!(@str Logger, log, logger_log);

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    exports.create_named_method("install", install)?;

    // COUNTER
    let counter = [
        Property::new(&env, "isActive")?.with_method(counter_is_active),
        Property::new(&env, "inc")?.with_method(counter_inc),
    ];
    let counter_class = env.define_class("Counter", counter_constructor, &counter)?;
    exports.set_named_property("Counter", counter_class)?;

    // GAUGE
    let gauge = [
        Property::new(&env, "isActive")?.with_method(gauge_is_active),
        Property::new(&env, "set")?.with_method(gauge_set),
    ];
    let gauge_class = env.define_class("Gauge", gauge_constructor, &gauge)?;
    exports.set_named_property("Gauge", gauge_class)?;

    // HISTOGRAM
    let histogram = [
        Property::new(&env, "isActive")?.with_method(histogram_is_active),
        Property::new(&env, "add")?.with_method(histogram_add),
    ];
    let histogram_class = env.define_class("Histogram", histogram_constructor, &histogram)?;
    exports.set_named_property("Histogram", histogram_class)?;

    // PULSE
    let pulse_props = [
        Property::new(&env, "isActive")?.with_method(pulse_is_active),
        Property::new(&env, "inc")?.with_method(pulse_inc),
        Property::new(&env, "dec")?.with_method(pulse_dec),
        Property::new(&env, "set")?.with_method(pulse_set),
    ];
    let pulse_class = env.define_class("Pulse", pulse_constructor, &pulse_props)?;
    exports.set_named_property("Pulse", pulse_class)?;

    // DICT
    let dict = [
        Property::new(&env, "isActive")?.with_method(dict_is_active),
        Property::new(&env, "set")?.with_method(dict_set),
    ];
    let dict_class = env.define_class("Dict", dict_constructor, &dict)?;
    exports.set_named_property("Dict", dict_class)?;

    // LOGGER
    let logger = [
        Property::new(&env, "isActive")?.with_method(logger_is_active),
        Property::new(&env, "log")?.with_method(logger_log),
    ];
    let logger_class = env.define_class("Logger", logger_constructor, &logger)?;
    exports.set_named_property("Logger", logger_class)?;

    Ok(())
}
