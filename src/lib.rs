use derive_more::{Deref, DerefMut};
use napi::{
    CallContext, Env, Error, JsBoolean, JsNumber, JsObject, JsString, JsUndefined, Property, Result,
};
use napi_derive::{js_function, module_exports};
use once_cell::sync::OnceCell;
use rillrate::{Col, Counter, Dict, Gauge, Histogram, Logger, Pulse, RillRate, Row, Table};
use std::convert::TryInto;

static RILLRATE: OnceCell<RillRate> = OnceCell::new();

fn js_err(reason: impl ToString) -> Error {
    Error::from_reason(reason.to_string())
}

trait IntoJs<T> {
    fn into_js(self, ctx: &CallContext) -> Result<T>;
}

impl IntoJs<JsUndefined> for () {
    fn into_js(self, ctx: &CallContext) -> Result<JsUndefined> {
        ctx.env.get_undefined()
    }
}

trait FromJs: Sized {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self>;
}

impl FromJs for String {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        ctx.get::<JsString>(idx)?.into_utf8()?.into_owned()
    }
}

impl FromJs for f64 {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        ctx.get::<JsNumber>(idx)?.try_into()
    }
}

impl FromJs for u32 {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        ctx.get::<JsNumber>(idx)?.try_into()
    }
}

impl FromJs for Row {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        u32::from_js(ctx, idx).map(|row| Row(row as u64))
    }
}

impl FromJs for Vec<f64> {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        let obj = ctx.get::<JsObject>(idx)?;
        let len = obj.get_array_length()?;
        let mut items = Vec::new();
        for idx in 0..len {
            let value: f64 = obj.get_element::<JsNumber>(idx)?.try_into()?;
            items.push(value);
        }
        Ok(items)
    }
}

impl FromJs for Vec<(Col, String)> {
    fn from_js(ctx: &CallContext, idx: usize) -> Result<Self> {
        let obj = ctx.get::<JsObject>(idx)?;
        let len = obj.get_array_length()?;
        let mut items = Vec::new();
        for idx in 0..len {
            let item = obj.get_element::<JsObject>(idx)?;
            let value: u32 = item.get_element::<JsNumber>(0)?.try_into()?;
            let s: String = item
                .get_element::<JsString>(idx)?
                .into_utf8()?
                .into_owned()?;
            items.push((Col(value as u64), s));
        }
        Ok(items)
    }
}

/// The normal `CallContext` that is have to be.
#[derive(Deref, DerefMut)]
struct Context<'a> {
    ctx: CallContext<'a>,
}

impl<'a> Context<'a> {
    fn wrap(ctx: CallContext<'a>) -> Self {
        Self { ctx }
    }

    fn from_js<T: FromJs>(&self, idx: usize) -> Result<T> {
        T::from_js(&self.ctx, idx)
    }

    fn into_js<T: IntoJs<J>, J>(&self, value: T) -> Result<J> {
        value.into_js(&self.ctx)
    }

    fn this_as<T: 'static>(&self) -> Result<&T> {
        let this: JsObject = self.ctx.this_unchecked();
        let projection: &mut T = self.ctx.env.unwrap(&this)?;
        Ok(projection)
    }

    fn assign<T: 'static>(&self, instance: T) -> Result<()> {
        let mut this: JsObject = self.ctx.this_unchecked();
        self.ctx.env.wrap(&mut this, instance)?;
        Ok(())
    }
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
            let ctx = Context::wrap(ctx);
            let arg0: String = ctx.from_js(0)?;
            let instance = $cls::create(&arg0).map_err(js_err)?;
            ctx.assign(instance)?;
            ctx.env.get_undefined()
        }
    };

    ($cls:ident :: $meth:ident [ $tot:expr ] ( $( $arg_ty:ty ),* ) as $name:ident -> $res_ty:ty) => {
        #[js_function($tot)]
        fn $name(ctx: CallContext) -> Result<$res_ty> {
            let ctx = Context::wrap(ctx);
            let mut counter = 0;
            let mut next = || {
                let last = counter;
                counter += 1;
                last
            };
            let provider = ctx.this_as::<$cls>()?;
            let res = provider.$meth(
                $(
                    <$arg_ty>::from_js(&ctx.ctx, next())?,
                ),*
            );
            ctx.into_js(res)
        }
    };

    (@bool $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsBoolean> {
            let ctx = Context::wrap(ctx);
            let provider = ctx.this_as::<$cls>()?;
            ctx.env.get_boolean(provider.$meth())
        }
    };

    (@f64 $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let ctx = Context::wrap(ctx);
            let arg0: f64 = ctx.from_js(0)?;
            let provider = ctx.this_as::<$cls>()?;
            provider.$meth(arg0);
            ctx.env.get_undefined()
        }
    };

    (@str $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(1)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let ctx = Context::wrap(ctx);
            let arg0: String = ctx.from_js(0)?;
            let provider = ctx.this_as::<$cls>()?;
            provider.$meth(arg0);
            ctx.env.get_undefined()
        }
    };

    (@two_str $cls:ident, $meth:ident, $name:ident) => {
        #[js_function(2)]
        fn $name(ctx: CallContext) -> Result<JsUndefined> {
            let ctx = Context::wrap(ctx);
            let arg0: String = ctx.from_js(0)?;
            let arg1: String = ctx.from_js(1)?;
            let provider = ctx.this_as::<$cls>()?;
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
    let ctx = Context::wrap(ctx);
    let arg0: String = ctx.from_js(0)?;
    let arg1: f64 = ctx.from_js(1)?;
    let arg2: f64 = ctx.from_js(2)?;
    let instance = Gauge::create(&arg0, arg1, arg2).map_err(js_err)?;
    ctx.assign(instance)?;
    ctx.env.get_undefined()
}
js_decl!(@bool Gauge, is_active, gauge_is_active);
js_decl!(@f64 Gauge, set, gauge_set);

#[js_function(1)]
fn pulse_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let ctx = Context::wrap(ctx);
    let arg0: String = ctx.from_js(0)?;
    // TODO: Try to get depth from args
    let instance = Pulse::create(&arg0, None).map_err(js_err)?;
    ctx.assign(instance)?;
    ctx.env.get_undefined()
}
js_decl!(@bool Pulse, is_active, pulse_is_active);
js_decl!(@f64 Pulse, inc, pulse_inc);
js_decl!(@f64 Pulse, dec, pulse_dec);
js_decl!(@f64 Pulse, set, pulse_set);

#[js_function(2)]
fn histogram_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let ctx = Context::wrap(ctx);
    let arg0: String = ctx.from_js(0)?;
    let arg1: Vec<f64> = ctx.from_js(1)?;
    let instance = Histogram::create(&arg0, &arg1).map_err(js_err)?;
    ctx.assign(instance)?;
    ctx.env.get_undefined()
}
js_decl!(@bool Histogram, is_active, histogram_is_active);
js_decl!(Histogram::add[1](f64) as histogram_add -> JsUndefined);

js_decl!(@new Dict, dict_constructor);
js_decl!(@bool Dict, is_active, dict_is_active);
js_decl!(@two_str Dict, set, dict_set);

js_decl!(@new Logger, logger_constructor);
js_decl!(@bool Logger, is_active, logger_is_active);
js_decl!(Logger::log[1](String) as logger_log -> JsUndefined);

#[js_function(2)]
fn table_constructor(ctx: CallContext) -> Result<JsUndefined> {
    let ctx = Context::wrap(ctx);
    let arg0: String = ctx.from_js(0)?;
    let arg1: Vec<(Col, String)> = ctx.from_js(1)?;
    let instance = Table::create(&arg0, arg1).map_err(js_err)?;
    ctx.assign(instance)?;
    ctx.env.get_undefined()
}
js_decl!(@bool Table, is_active, table_is_active);
js_decl!(Table::add_row[1](Row) as table_add_row -> JsUndefined);
js_decl!(Table::del_row[1](Row) as table_del_row -> JsUndefined);

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

    // TABLE
    let table = [
        Property::new(&env, "isActive")?.with_method(table_is_active),
        Property::new(&env, "add_row")?.with_method(table_add_row),
        Property::new(&env, "del_row")?.with_method(table_del_row),
        /*
        Property::new(&env, "set_cell")?.with_method(table_set_cell),
        */
    ];
    let table_class = env.define_class("Table", table_constructor, &table)?;
    exports.set_named_property("Table", table_class)?;

    Ok(())
}
