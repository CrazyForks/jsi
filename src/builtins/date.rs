use std::time::{SystemTime, UNIX_EPOCH};

use crate::ast_node::CallContext;
use crate::builtins::function::builtin_function;
use crate::builtins::global::get_global_object_by_name;
use crate::constants::GLOBAL_DATE_NAME;
use crate::context::Context;
use crate::error::JSIResult;
use crate::value::Value;

fn date_now(_call_ctx: &mut CallContext, _args: Vec<Value>) -> JSIResult<Value> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64;
    Ok(Value::Number(now))
}

pub fn bind_global_date(ctx: &mut Context) {
    let date_rc = get_global_object_by_name(ctx, GLOBAL_DATE_NAME);
    let mut date = date_rc.borrow_mut();

    let now_name = String::from("now");
    date.property.insert(now_name.clone(), crate::builtins::object::Property {
        enumerable: true,
        value: builtin_function(ctx, now_name, 0f64, date_now),
    });
}
