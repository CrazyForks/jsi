use std::cell::{RefCell};
use std::rc::{Rc};

use crate::ast_node::{ClassType, CallContext};
use crate::builtins::promise::bind_global_promise;
use crate::builtins::regexp::bind_global_regexp;
use crate::constants::{GLOBAL_OBJECT_NAME_LIST, GLOBAL_OBJECT_NAME, PROTO_PROPERTY_NAME, GLOBAL_ERROR_NAME, GLOBAL_TYPE_ERROR_NAME, GLOBAL_JSON_NAME, GLOBAL_DATE_NAME};
use crate::scope::get_global_scope;
use crate::value::Value;
use crate::context::{Context};
use crate::error::{JSIResult};
use super::array::bind_global_array;
use super::boolean::{bind_global_boolean};
use super::error::{bind_global_error};
use super::function::{bind_global_function, builtin_function};
use super::number::bind_global_number;
use super::object::{Object, Property, bind_global_object};
use super::string::bind_global_string;
use super::json::bind_global_json;
use super::date::bind_global_date;

pub const IS_GLOABL_OBJECT: &str = "isGlobal";

pub fn new_global_object() -> Rc<RefCell<Object>> {
  let object = Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let object_clone = Rc::clone(&object);
  let mut object_mut = (*object_clone).borrow_mut();

  // 创建原型对象 prototype
  // Object.prototype 是所有对象的原型
  // 原型上面的方法，通过 bind_global_object 挂载
  let prototype =  Rc::new(RefCell::new(Object::new(ClassType::Object, None)));
  let prototype_clone = Rc::clone(&prototype);
  let mut prototype_mut = prototype_clone.borrow_mut();
  prototype_mut.define_property(String::from("constructor"), Property {
    enumerable: false,
    value: Value::RefObject(Rc::downgrade(&object)),
  });
  object_mut.prototype = Some(prototype);
  object
}

// 全局对象
pub fn new_global_this() -> Rc<RefCell<Object>> {
  // 先创建全局 Object，以及 Object.prototype
  let first_obj = new_global_object();
  let first_obj_clone = Rc::clone(&first_obj);
  let mut first_obj_borrow = (*first_obj_clone).borrow_mut();
  first_obj_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
  first_obj_borrow.set_inner_property_value(String::from("name"), Value::String(GLOBAL_OBJECT_NAME.to_string()));
  // native function
  let native_function = new_global_object();
  {
    let native_function_rc = Rc::clone(&native_function);
    let mut native_borrow = native_function_rc.borrow_mut();
    // 绑定 native_function的原型到全局 Object.prototype
    // Object['__proto__'] === native_function
    first_obj_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&native_function)));
    // native_function.__proto__ === Object['__proto__'].__proto__ === Object.prototype
    if let Some(prop) = &first_obj_borrow.prototype {
      native_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(prop)));
    }
  }
  
  // Global
  let global = new_global_object();
  let global_clone = Rc::clone(&global);
  {
    let mut global_obj = global_clone.borrow_mut();
    global_obj.property.insert(GLOBAL_OBJECT_NAME.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&first_obj))});
    // 创建并绑定全局对象
    for name in GLOBAL_OBJECT_NAME_LIST.iter() {
      if name == &GLOBAL_OBJECT_NAME {
        continue;
      }
      let object = new_global_object();
      let object_rc = Rc::clone(&object);
      let mut object_borrow = object_rc.borrow_mut();
      // 绑定当前对象的原型
      object_borrow.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::Object(Rc::clone(&native_function)));

      // 标记是全局对象
      object_borrow.set_inner_property_value(IS_GLOABL_OBJECT.to_string(), Value::Boolean(true));
      // 添加对象 name
      object_borrow.set_inner_property_value(String::from("name"), Value::String(name.to_string()));
      global_obj.property.insert(name.to_string(), Property { enumerable: true, value: Value::Object(Rc::clone(&object))});
    }
  }
  
  return global;
}

pub fn bind_global(ctx: &mut Context) {
  // 先绑定全局函数 parseInt 和 parseFloat（不依赖于其他全局对象）
  bind_global_functions(ctx);

  // 绑定 Object 的 静态方法 和 原型链方法
  bind_global_object(ctx);
  // 绑定 Function 的 静态方法 和 原型链方法
  bind_global_function(ctx);
  // 绑定 Array 的 静态方法 和 原型链方法
  bind_global_array(ctx);
  // 绑定  String 的 静态方法 和 原型链方法
  bind_global_string(ctx);
  // 绑定  Boolean 的 静态方法 和 原型链方法
  bind_global_boolean(ctx);
  // 绑定  Number 的 静态方法 和 原型链方法
  bind_global_number(ctx);

  // 绑定 Promise 的 静态方法 和 原型链方法
  bind_global_promise(ctx);
  // 绑定 RegExp 的 静态方法 和 原型链方法
  bind_global_regexp(ctx);
  // 绑定 JSON 的 静态方法
  bind_global_json(ctx);
  // 绑定 Date 的 静态方法
  bind_global_date(ctx);
  // 绑定  Error 的 静态方法 和 原型链方法
  bind_global_error(ctx, GLOBAL_ERROR_NAME);
  bind_global_error(ctx, GLOBAL_TYPE_ERROR_NAME);

  let obj_rc = get_global_object(ctx, GLOBAL_OBJECT_NAME.to_string());
  let obj_rc =  obj_rc.borrow();
  let obj_prototype_rc = &obj_rc.prototype;
  if let Some(obj_prototype) = obj_prototype_rc {
    // 绑定 prototype.[[Property]]
    for name in GLOBAL_OBJECT_NAME_LIST.iter() {
      if name == &GLOBAL_OBJECT_NAME {
        continue;
      }
      let global_item_rc =  get_global_object(ctx, name.to_string());
      let global_item_ref = global_item_rc.borrow();
      if let Some(prop)= &global_item_ref.prototype {

        let prototype_rc = Rc::clone(prop);
        let mut prototype = (*prototype_rc).borrow_mut();

        // 除 Object 外，其他的原型对象的原型 [[Property]] 都是 Object 的原型对象
        prototype.set_inner_property_value(PROTO_PROPERTY_NAME.to_string(), Value::RefObject(Rc::downgrade(&obj_prototype)));

      }
    }
  }
}

pub fn get_global_object(ctx: &mut Context, name: String) -> Rc<RefCell<Object>> {

  let value = {
    let clone_global_mut = ctx.global.borrow_mut();
    clone_global_mut.get_value(name.clone())
  };

  let obj = value.to_object(ctx);
  return obj;
}

pub fn get_global_object_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
  let value = {
    let clone_global_mut = ctx.global.borrow_mut();
    clone_global_mut.get_value(name.to_string().clone())
  };
  let obj = value.to_object(ctx);
  return obj;
}
// 获取全局对象的 prototype
pub fn get_global_object_prototype_by_name(ctx: &mut Context, name: &str) -> Rc<RefCell<Object>> {
  let obj = get_global_object_by_name(ctx, name);
  let obj_clone = Rc::clone(&obj);
  let obj_borrow = obj_clone.borrow_mut();
  let proto = (obj_borrow.prototype.as_ref()).unwrap();
  return Rc::clone(&proto);
}

// 绑定全局函数 parseInt 和 parseFloat
fn bind_global_functions(ctx: &mut Context) {
  // 先创建所有函数（必须在借用global之前创建，避免双重借用）
  let parse_int_fun = builtin_function(ctx, String::from("parseInt"), 2f64, global_parse_int);
  let parse_float_fun = builtin_function(ctx, String::from("parseFloat"), 1f64, global_parse_float);
  let is_nan_fun = builtin_function(ctx, String::from("isNaN"), 1f64, global_is_nan);
  let is_finite_fun = builtin_function(ctx, String::from("isFinite"), 1f64, global_is_finite);

  // URI 函数
  let decode_uri_fun = builtin_function(ctx, String::from("decodeURI"), 1f64, global_decode_uri);
  let decode_uri_component_fun = builtin_function(ctx, String::from("decodeURIComponent"), 1f64, global_decode_uri_component);
  let encode_uri_fun = builtin_function(ctx, String::from("encodeURI"), 1f64, global_encode_uri);
  let encode_uri_component_fun = builtin_function(ctx, String::from("encodeURIComponent"), 1f64, global_encode_uri_component);

  // 借用 global 并插入函数到全局对象
  let global_this = Rc::clone(&ctx.global);
  let mut global_mut = global_this.borrow_mut();

  // parseInt(string, radix)
  global_mut.property.insert(String::from("parseInt"), Property { enumerable: true, value: parse_int_fun.clone() });

  // parseFloat(string)
  global_mut.property.insert(String::from("parseFloat"), Property { enumerable: true, value: parse_float_fun.clone() });

  // isNaN(value)
  global_mut.property.insert(String::from("isNaN"), Property { enumerable: true, value: is_nan_fun.clone() });

  // isFinite(value)
  global_mut.property.insert(String::from("isFinite"), Property { enumerable: true, value: is_finite_fun.clone() });

  // NaN 常量
  global_mut.property.insert(String::from("NaN"), Property { enumerable: true, value: Value::NAN });

  // Infinity 常量
  global_mut.property.insert(String::from("Infinity"), Property { enumerable: true, value: Value::Number(f64::INFINITY) });

  global_mut.property.insert(String::from("decodeURI"), Property { enumerable: true, value: decode_uri_fun.clone() });
  global_mut.property.insert(String::from("decodeURIComponent"), Property { enumerable: true, value: decode_uri_component_fun.clone() });
  global_mut.property.insert(String::from("encodeURI"), Property { enumerable: true, value: encode_uri_fun.clone() });
  global_mut.property.insert(String::from("encodeURIComponent"), Property { enumerable: true, value: encode_uri_component_fun.clone() });

  // 释放 global 的借用
  drop(global_mut);

  // 同时把这些函数添加到全局作用域，以便变量查找能找到它们
  let global_scope = get_global_scope(Rc::clone(&ctx.cur_scope));
  let mut scope_mut = global_scope.borrow_mut();
  scope_mut.set_value(String::from("parseInt"), parse_int_fun, false);
  scope_mut.set_value(String::from("parseFloat"), parse_float_fun, false);
  scope_mut.set_value(String::from("isNaN"), is_nan_fun, false);
  scope_mut.set_value(String::from("isFinite"), is_finite_fun, false);
  scope_mut.set_value(String::from("NaN"), Value::NAN, false);
  scope_mut.set_value(String::from("Infinity"), Value::Number(f64::INFINITY), false);
  scope_mut.set_value(String::from("decodeURI"), decode_uri_fun, false);
  scope_mut.set_value(String::from("decodeURIComponent"), decode_uri_component_fun, false);
  scope_mut.set_value(String::from("encodeURI"), encode_uri_fun, false);
  scope_mut.set_value(String::from("encodeURIComponent"), encode_uri_component_fun, false);
}

// parseInt(string, radix)
// 解析字符串为整数
fn global_parse_int(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // 获取要解析的字符串
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::NAN);
  };

  // 获取 radix (基数)
  let radix: i32 = if args.len() > 1 {
    if let Some(r) = args[1].to_number(call_ctx.ctx) {
      r as i32
    } else {
      10
    }
  } else {
    // 默认基数，根据字符串前缀判断
    if input_str.starts_with("0x") || input_str.starts_with("0X") {
      16
    } else {
      10
    }
  };

  // radix 必须在 2-36 之间
  if radix < 2 || radix > 36 {
    return Ok(Value::NAN);
  }

  // 去除前导空白
  let trimmed = input_str.trim();

  // 处理十六进制前缀
  let parse_str = if radix == 16 {
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
      &trimmed[2..]
    } else {
      trimmed
    }
  } else if radix == 8 && trimmed.starts_with("0") {
    // 处理八进制前缀（ES1 风格）
    &trimmed[1..]
  } else {
    trimmed
  };

  // 处理符号
  let (sign, parse_str) = if parse_str.starts_with('-') {
    (-1.0, &parse_str[1..])
  } else if parse_str.starts_with('+') {
    (1.0, &parse_str[1..])
  } else {
    (1.0, parse_str)
  };

  // 解析数字
  let mut result: f64 = 0f64;
  let mut parsed_any = false;

  for c in parse_str.chars() {
    let digit_value = if c >= '0' && c <= '9' {
      (c as i32) - ('0' as i32)
    } else if c >= 'a' && c <= 'z' {
      (c as i32) - ('a' as i32) + 10
    } else if c >= 'A' && c <= 'Z' {
      (c as i32) - ('A' as i32) + 10
    } else {
      // 非数字字符，停止解析
      break;
    };

    // 检查 digit 是否有效（在 radix 范围内）
    if digit_value >= radix {
      break;
    }

    result = result * (radix as f64) + (digit_value as f64);
    parsed_any = true;
  }

  if !parsed_any {
    return Ok(Value::NAN);
  }

  Ok(Value::Number(result * sign))
}

// parseFloat(string)
// 解析字符串为浮点数
fn global_parse_float(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // 获取要解析的字符串
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::NAN);
  };

  // 去除前导空白
  let trimmed = input_str.trim();

  // 处理符号
  let (sign, parse_str) = if trimmed.starts_with('-') {
    (-1.0, &trimmed[1..])
  } else if trimmed.starts_with('+') {
    (1.0, &trimmed[1..])
  } else {
    (1.0, trimmed)
  };

  // 解析浮点数
  let mut result: f64 = 0f64;
  let mut parsed_any = false;
  let mut decimal_position: f64 = 0f64;
  let mut in_decimal = false;
  let mut in_exponent = false;
  let mut exponent_sign: f64 = 1.0;
  let mut exponent_value: f64 = 0f64;

  for c in parse_str.chars() {
    if c >= '0' && c <= '9' {
      let digit = (c as i32 - '0' as i32) as f64;
      if in_exponent {
        exponent_value = exponent_value * 10.0 + digit;
      } else if in_decimal {
        decimal_position += 1.0;
        result = result + digit / (10f64.powf(decimal_position));
      } else {
        result = result * 10.0 + digit;
      }
      parsed_any = true;
    } else if c == '.' && !in_decimal && !in_exponent {
      in_decimal = true;
    } else if (c == 'e' || c == 'E') && parsed_any && !in_exponent {
      in_exponent = true;
    } else if c == '-' && in_exponent && exponent_value == 0f64 {
      exponent_sign = -1.0;
    } else if c == '+' && in_exponent && exponent_value == 0f64 {
      exponent_sign = 1.0;
    } else {
      // 非数字字符，停止解析
      break;
    }
  }

  if !parsed_any {
    return Ok(Value::NAN);
  }

  // 应用指数
  result = result * sign * 10f64.powf(exponent_value * exponent_sign);

  Ok(Value::Number(result))
}

// isNaN(value)
// 检查值是否为 NaN
fn global_is_nan(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // 获取要检查的值
  let value = if args.len() > 0 {
    args[0].clone()
  } else {
    return Ok(Value::Boolean(false));
  };

  // 先检查是否为 Value::NAN 类型
  if value.is_nan() {
    return Ok(Value::Boolean(true));
  }

  // 将值转换为数字，然后检查是否为 NaN
  let num_value = value.to_number(call_ctx.ctx);

  // to_number 返回 Option<f64>，需要检查转换后的数字是否为 NaN
  let is_nan = match num_value {
    None => true,
    Some(n) => n.is_nan(),
  };

  Ok(Value::Boolean(is_nan))
}

// isFinite(value)
// 检查值是否为有限数（非 NaN、非 Infinity、非 -Infinity）
fn global_is_finite(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  // 获取要检查的值
  let value = if args.len() > 0 {
    args[0].clone()
  } else {
    return Ok(Value::Boolean(false));
  };

  // 先检查是否为 Value::NAN 类型
  if value.is_nan() {
    return Ok(Value::Boolean(false));
  }

  // 检查是否为 Infinity
  if value.is_infinity() {
    return Ok(Value::Boolean(false));
  }

  // 将值转换为数字
  let num_value = value.to_number(call_ctx.ctx);

  // 检查转换后的数字是否为有限数
  let is_finite = match num_value {
    None => false,
    Some(n) => n.is_finite(),
  };

  Ok(Value::Boolean(is_finite))
}

// decodeURI(encodedURI)
// 解码 URI，保留 URI 特殊字符
fn global_decode_uri(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::String(String::from("undefined")));
  };

  let result = decode_uri_helper(&input_str, false);
  Ok(Value::String(result))
}

// decodeURIComponent(encodedURIComponent)
// 解码 URI 组件，解码所有特殊字符
fn global_decode_uri_component(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::String(String::from("undefined")));
  };

  let result = decode_uri_helper(&input_str, true);
  Ok(Value::String(result))
}

// encodeURI(uri)
// 编码 URI，保留 URI 特殊字符
fn global_encode_uri(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::String(String::from("undefined")));
  };

  let result = encode_uri_helper(&input_str, false);
  Ok(Value::String(result))
}

// encodeURIComponent(uriComponent)
// 编码 URI 组件，编码所有特殊字符
fn global_encode_uri_component(call_ctx: &mut CallContext, args: Vec<Value>) -> JSIResult<Value> {
  let input_str = if args.len() > 0 {
    args[0].to_string(call_ctx.ctx)
  } else {
    return Ok(Value::String(String::from("undefined")));
  };

  let result = encode_uri_helper(&input_str, true);
  Ok(Value::String(result))
}

// 辅助函数：判断字符是否是 URI 保留字符
fn is_uri_reserved(c: char) -> bool {
  matches!(c, ';' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | ',')
}

// 辅助函数：判断字符是否是 URI 非转义字符
fn is_uri_unreserved(c: char) -> bool {
  (c >= 'A' && c <= 'Z') ||
  (c >= 'a' && c <= 'z') ||
  (c >= '0' && c <= '9') ||
  matches!(c, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
}

// 辅助函数：解码 URI
fn decode_uri_helper(input: &str, is_component: bool) -> String {
  let mut result = String::new();
  let mut chars = input.chars().peekable();

  while let Some(c) = chars.next() {
    if c == '%' {
      let mut hex = String::new();
      if let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
        let hex_str = format!("{}{}", h1, h2);
        if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
          // 如果不是 component 模式下，保留字符不解码
          if !is_component && is_uri_reserved(byte as char) {
            result.push('%');
            result.push(h1);
            result.push(h2);
          } else {
            result.push(byte as char);
          }
        } else {
          result.push('%');
          result.push(h1);
          result.push(h2);
        }
      } else {
        result.push('%');
      }
    } else {
      result.push(c);
    }
  }

  result
}

// 辅助函数：编码 URI
fn encode_uri_helper(input: &str, is_component: bool) -> String {
  let mut result = String::new();

  for c in input.chars() {
    if is_uri_unreserved(c) {
      result.push(c);
    } else if !is_component && (is_uri_reserved(c) || c == '#') {
      // encodeURI 不编码保留字符和 #
      result.push(c);
    } else {
      // 需要编码的字符
      let bytes = c.to_string().into_bytes();
      for byte in bytes {
        result.push_str(&format!("%{:02X}", byte));
      }
    }
  }

  result
}