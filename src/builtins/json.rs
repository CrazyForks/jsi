use std::cell::RefCell;
use std::rc::Rc;
use serde_json;

use crate::ast_node::{CallContext, ClassType};
use crate::builtins::function::builtin_function;
use crate::builtins::global::get_global_object_by_name;
use crate::builtins::object::{create_object, Object, Property};
use crate::constants::{GLOBAL_JSON_NAME, PROTO_PROPERTY_NAME};
use crate::context::Context;
use crate::error::{JSIResult, JSIError, JSIErrorType};
use crate::value::Value;

fn json_value_to_js_value(ctx: &mut Context, value: &serde_json::Value) -> JSIResult<Value> {
    match value {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(Value::Number(f))
            } else if let Some(i) = n.as_i64() {
                Ok(Value::Number(i as f64))
            } else {
                Ok(Value::Number(0f64))
            }
        }
        serde_json::Value::String(s) => Ok(Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                result.push(json_value_to_js_value(ctx, item)?);
            }
            Ok(create_array_from_values(ctx, result))
        }
        serde_json::Value::Object(obj) => {
            let obj_rc = create_object(ctx, ClassType::Object, None);
            {
                let mut obj_mut = obj_rc.borrow_mut();
                for (key, value) in obj {
                    let js_value = json_value_to_js_value(ctx, value)?;
                    obj_mut.define_property(key.clone(), Property {
                        enumerable: true,
                        value: js_value,
                    });
                }
            }
            Ok(Value::Object(obj_rc))
        }
    }
}

fn create_array_from_values(ctx: &mut Context, values: Vec<Value>) -> Value {
    let global_array = get_global_object_by_name(ctx, "Array");
    let array = create_object(ctx, ClassType::Array, None);
    let array_clone = Rc::clone(&array);
    let mut array_mut = (*array_clone).borrow_mut();

    array_mut.define_property(String::from("length"), Property {
        enumerable: false,
        value: Value::Number(values.len() as f64),
    });

    let global_prototype = get_global_object_prototype_by_name(ctx, "Array");
    array_mut.set_inner_property_value(
        PROTO_PROPERTY_NAME.to_string(),
        Value::RefObject(Rc::downgrade(&global_prototype)),
    );
    array_mut.constructor = Some(Rc::downgrade(&global_array));

    for (index, value) in values.iter().enumerate() {
        array_mut.define_property(index.to_string(), Property {
            enumerable: true,
            value: value.clone(),
        });
    }

    Value::Array(array)
}

fn get_global_object_prototype_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
    let obj = get_global_object_by_name(ctx, name);
    let obj_borrow = obj.borrow();
    if let Some(prototype) = &obj_borrow.prototype {
        Rc::clone(prototype)
    } else {
        panic!("{} prototype not found", name);
    }
}

fn json_parse(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    if args.is_empty() {
        return Err(JSIError::new(JSIErrorType::SyntaxError, String::from("Unexpected end of JSON input"), 0, 0));
    }

    let json_str = args[0].to_string(call_ctx.ctx);
    match serde_json::from_str::<serde_json::Value>(&json_str) {
        Ok(value) => json_value_to_js_value(call_ctx.ctx, &value),
        Err(e) => Err(JSIError::new(JSIErrorType::SyntaxError, e.to_string(), 0, 0)),
    }
}

fn js_value_to_json_value_impl(
    ctx: &mut Context,
    value: &Value,
    visited: &mut std::collections::HashSet<usize>,
) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Undefined => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Number(n) => {
            if *n == n.floor() && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                serde_json::Value::Number(serde_json::Number::from(*n as i64))
            } else {
                serde_json::Number::from_f64(*n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            }
        }
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => {
            let arr_ptr = Rc::as_ptr(arr) as usize;
            if visited.contains(&arr_ptr) {
                // 检测到循环引用，返回 null（符合 JSON.stringify 的行为）
                return serde_json::Value::Null;
            }
            visited.insert(arr_ptr);

            let arr_borrow = arr.borrow();
            let mut result = Vec::new();
            if let Value::Number(len) = arr_borrow.get_value(String::from("length")) {
                for i in 0..(len as usize) {
                    let item = arr_borrow.get_value(i.to_string());
                    result.push(js_value_to_json_value_impl(ctx, &item, visited));
                }
            }
            visited.remove(&arr_ptr);
            serde_json::Value::Array(result)
        }
        Value::Object(obj) => {
            let obj_ptr = Rc::as_ptr(obj) as usize;
            if visited.contains(&obj_ptr) {
                // 检测到循环引用，返回 null
                return serde_json::Value::Null;
            }
            visited.insert(obj_ptr);

            let obj_borrow = obj.borrow();
            let mut map = serde_json::Map::new();
            for key in obj_borrow.property_list.iter() {
                let value = obj_borrow.get_value(key.clone());
                let json_value = js_value_to_json_value_impl(ctx, &value, visited);
                map.insert(key.clone(), json_value);
            }
            visited.remove(&obj_ptr);
            serde_json::Value::Object(map)
        }
        Value::Function(_) => serde_json::Value::Null,
        Value::RefObject(weak_obj) => {
            if let Some(obj) = weak_obj.upgrade() {
                let obj_ptr = Rc::as_ptr(&obj) as usize;
                if visited.contains(&obj_ptr) {
                    return serde_json::Value::Null;
                }
                visited.insert(obj_ptr);
                let obj_borrow = obj.borrow();
                let mut map = serde_json::Map::new();
                for key in obj_borrow.property_list.iter() {
                    let value = obj_borrow.get_value(key.clone());
                    let json_value = js_value_to_json_value_impl(ctx, &value, visited);
                    map.insert(key.clone(), json_value);
                }
                visited.remove(&obj_ptr);
                serde_json::Value::Object(map)
            } else {
                serde_json::Value::Null
            }
        }
        Value::Scope(_) => serde_json::Value::Null,
        Value::Interrupt(_, _) => serde_json::Value::Null,
        Value::ByteCode(_) => serde_json::Value::Null,
        Value::StringObj(_) => serde_json::Value::Null,
        Value::NumberObj(_) => serde_json::Value::Null,
        Value::BooleanObj(_) => serde_json::Value::Null,
        Value::Promise(_) => serde_json::Value::Null,
        Value::NAN => serde_json::Value::Null,
    }
}

fn js_value_to_json_value(ctx: &mut Context, value: &Value) -> serde_json::Value {
    let mut visited = std::collections::HashSet::new();
    js_value_to_json_value_impl(ctx, value, &mut visited)
}

fn json_stringify(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let json_value = js_value_to_json_value(call_ctx.ctx, &args[0]);
    Ok(Value::String(json_value.to_string()))
}

pub fn bind_global_json(ctx: &mut Context) {
    let json_rc = get_global_object_by_name(ctx, GLOBAL_JSON_NAME);
    let mut json = (*json_rc).borrow_mut();

    let parse_name = String::from("parse");
    json.property.insert(parse_name.clone(), Property {
        enumerable: true,
        value: builtin_function(ctx, parse_name, 1f64, json_parse),
    });

    let stringify_name = String::from("stringify");
    json.property.insert(stringify_name.clone(), Property {
        enumerable: true,
        value: builtin_function(ctx, stringify_name, 1f64, json_stringify),
    });
}
