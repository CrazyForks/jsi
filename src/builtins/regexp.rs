use std::{rc::Rc, cell::RefCell};
use crate::constants::{GLOBAL_REGEXP_NAME, PROTO_PROPERTY_NAME};
use crate::context::{Context};
use crate::{value::{Value, INSTANTIATE_OBJECT_METHOD_NAME}, ast_node::{ClassType, CallContext}, error::JSIResult};

use super::global::{get_global_object_prototype_by_name, get_global_object_by_name};
use super::{object::{create_object, Object, Property}, function::builtin_function};

pub fn create_regexp(ctx: &mut Context, pattern: Value, flags: Option<Value>) -> Value {
    let global_regexp = get_global_object_by_name(ctx, GLOBAL_REGEXP_NAME);
    let regexp = create_object(ctx, ClassType::Object, None);
    let regexp_clone = Rc::clone(&regexp);
    let mut regexp_mut = (*regexp_clone).borrow_mut();

    regexp_mut.constructor = Some(Rc::downgrade(&global_regexp));

    let pattern_str = pattern.to_string(ctx);
    regexp_mut.define_property(String::from("source"), Property {
        enumerable: false,
        value: Value::String(pattern_str)
    });

    // 解析 flags
    let flags_str = if let Some(f) = flags {
        f.to_string(ctx)
    } else {
        String::from("")
    };

    regexp_mut.define_property(String::from("global"), Property {
        enumerable: false,
        value: Value::Boolean(flags_str.contains('g'))
    });
    regexp_mut.define_property(String::from("ignoreCase"), Property {
        enumerable: false,
        value: Value::Boolean(flags_str.contains('i'))
    });
    regexp_mut.define_property(String::from("multiline"), Property {
        enumerable: false,
        value: Value::Boolean(flags_str.contains('m'))
    });
    regexp_mut.define_property(String::from("lastIndex"), Property {
        enumerable: false,
        value: Value::Number(0f64)
    });

    let global_prototype = get_global_object_prototype_by_name(ctx, GLOBAL_REGEXP_NAME);
    regexp_mut.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&global_prototype)));

    Value::Object(regexp)
}

pub fn bind_global_regexp(ctx: &mut Context) {
    let regexp_rc = get_global_object_by_name(ctx, GLOBAL_REGEXP_NAME);
    let mut regexp = (*regexp_rc).borrow_mut();
    let create_function = builtin_function(ctx, INSTANTIATE_OBJECT_METHOD_NAME.to_string(), 2f64, create);
    regexp.set_inner_property_value(INSTANTIATE_OBJECT_METHOD_NAME.to_string(), create_function);

    if let Some(prop) = &regexp.prototype {
        let prototype_rc = Rc::clone(prop);
        let mut prototype = (*prototype_rc).borrow_mut();

        // RegExp.prototype.test
        let name = String::from("test");
        prototype.define_property(name.clone(), Property {
            enumerable: true,
            value: builtin_function(ctx, name, 1f64, test)
        });

        // RegExp.prototype.exec
        let name = String::from("exec");
        prototype.define_property(name.clone(), Property {
            enumerable: true,
            value: builtin_function(ctx, name, 1f64, exec)
        });

        // RegExp.prototype.toString
        let name = String::from("toString");
        prototype.define_property(name.clone(), Property {
            enumerable: true,
            value: builtin_function(ctx, name, 0f64, to_string)
        });
    }
}

// 简单的正则匹配 - 仅支持基本的字符串匹配
fn simple_match(pattern: &str, input: &str, ignore_case: bool) -> Option<Vec<String>> {
    let (search_pattern, search_input) = if ignore_case {
        (pattern.to_lowercase(), input.to_lowercase())
    } else {
        (pattern.to_string(), input.to_string())
    };

    if let Some(pos) = search_input.find(&search_pattern) {
        let mut captures = vec![];
        // 整个匹配
        let end = pos + search_pattern.len();
        captures.push(input[pos..end].to_string());
        Some(captures)
    } else {
        None
    }
}

// RegExp.prototype.test
fn test(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let this_obj = match &call_ctx.this {
        Value::Object(obj) => Rc::clone(obj),
        _ => return Ok(Value::Boolean(false))
    };

    let this = (*this_obj).borrow_mut();

    let source_prop = this.property.get("source");
    let pattern = if let Some(prop) = source_prop {
        prop.value.to_string(call_ctx.ctx)
    } else {
        String::from("")
    };

    let input = if args.len() > 0 {
        args[0].to_string(call_ctx.ctx)
    } else {
        String::from("")
    };

    let ignore_case = if let Some(prop) = this.property.get("ignoreCase") {
        matches!(prop.value, Value::Boolean(true))
    } else {
        false
    };

    let result = simple_match(&pattern, &input, ignore_case);
    Ok(Value::Boolean(result.is_some()))
}

// RegExp.prototype.exec
fn exec(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let this_obj = match &call_ctx.this {
        Value::Object(obj) => Rc::clone(obj),
        _ => return Ok(Value::Null)
    };

    let this = (*this_obj).borrow_mut();

    let source_prop = this.property.get("source");
    let pattern = if let Some(prop) = source_prop {
        prop.value.to_string(call_ctx.ctx)
    } else {
        String::from("")
    };

    let input = if args.len() > 0 {
        args[0].to_string(call_ctx.ctx)
    } else {
        String::from("")
    };

    let ignore_case = if let Some(prop) = this.property.get("ignoreCase") {
        matches!(prop.value, Value::Boolean(true))
    } else {
        false
    };

    if let Some(captures) = simple_match(&pattern, &input, ignore_case) {
        // 创建结果数组
        let result_arr = Rc::new(RefCell::new(Object::new(ClassType::Array, None)));
        {
            let mut result = result_arr.borrow_mut();
            let len = captures.len() as f64;
            result.define_property(String::from("length"), Property {
                enumerable: false,
                value: Value::Number(len)
            });

            for (i, cap) in captures.iter().enumerate() {
                result.define_property(i.to_string(), Property {
                    enumerable: true,
                    value: Value::String(cap.clone())
                });
            }

            // index 属性
            let search_input = if ignore_case {
                input.to_lowercase()
            } else {
                input.clone()
            };
            let search_pattern = if ignore_case {
                pattern.to_lowercase()
            } else {
                pattern.clone()
            };
            if let Some(pos) = search_input.find(&search_pattern) {
                result.define_property(String::from("index"), Property {
                    enumerable: false,
                    value: Value::Number(pos as f64)
                });
                result.define_property(String::from("input"), Property {
                    enumerable: false,
                    value: Value::String(input)
                });
            }
        }
        Ok(Value::Array(result_arr))
    } else {
        Ok(Value::Null)
    }
}

// RegExp.prototype.toString
fn to_string(call_ctx: &mut CallContext, _: Vec<Value>) -> JSIResult<Value> {
    let this_obj = match &call_ctx.this {
        Value::Object(obj) => Rc::clone(obj),
        _ => return Ok(Value::String(String::from("/(?:)/")))
    };

    let this = (*this_obj).borrow_mut();

    let source_prop = this.property.get("source");
    let source = if let Some(prop) = source_prop {
        prop.value.to_string(call_ctx.ctx)
    } else {
        String::from("")
    };

    let mut flags = String::new();
    if let Some(prop) = this.property.get("global") {
        if let Value::Boolean(true) = prop.value {
            flags.push('g');
        }
    }
    if let Some(prop) = this.property.get("ignoreCase") {
        if let Value::Boolean(true) = prop.value {
            flags.push('i');
        }
    }
    if let Some(prop) = this.property.get("multiline") {
        if let Value::Boolean(true) = prop.value {
            flags.push('m');
        }
    }

    Ok(Value::String(format!("/{}/{}", source, flags)))
}

fn create(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
    let pattern = if args.len() > 0 {
        args[0].clone()
    } else {
        Value::String(String::from(""))
    };

    let flags = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    Ok(create_regexp(call_ctx.ctx, pattern, flags))
}
