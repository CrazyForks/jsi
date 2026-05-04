use jsi::{JSI, value::Value};

// ============================================================================
// ES5 (ECMAScript 5) 核心语法测试
// ============================================================================

// ==================== 严格模式 (Strict Mode) ====================

// TODO: 修复严格模式下的栈溢出问题
// #[test]
// fn es5_strict_mode_this_undefined() {
//     let mut jsi = JSI::new();
//     jsi.set_strict(true);
//     let result = jsi.run(String::from(
//         "'use strict'; function f() { return this; } f()"
//     ));
//     let value = result.unwrap();
//     assert_eq!(value, Value::Undefined);
// }

#[test]
fn es5_non_strict_mode_this_global() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "function f() { return this; } typeof f()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("object")));
}

// ==================== JSON 对象 ====================

#[test]
fn es5_json_parse_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "JSON.parse('{\"name\":\"test\",\"value\":42}').value"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(42f64));
}

#[test]
fn es5_json_parse_string() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "JSON.parse('\"hello\"')"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("hello")));
}

#[test]
fn es5_json_parse_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "JSON.parse('[1,2,3]')[1]"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(2f64));
}

#[test]
fn es5_json_stringify_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var s = JSON.stringify({a:1}); s === '{\"a\":1}'"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn es5_json_stringify_circular_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var a = [1, 2]; a.push(a); JSON.stringify(a)"
    ));
    let value = result.unwrap();
    // 循环引用应该被序列化为 null
    if let Value::String(s) = value {
        assert!(s.contains("null"));
    } else {
        panic!("Expected string, got {:?}", value);
    }
}

#[test]
fn es5_json_stringify_circular_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var obj = {a: 1}; obj.self = obj; JSON.stringify(obj)"
    ));
    let value = result.unwrap();
    // 循环引用应该被序列化为 null
    if let Value::String(s) = value {
        assert!(s.contains("null"));
    } else {
        panic!("Expected string, got {:?}", value);
    }
}

#[test]
fn es5_json_stringify_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "JSON.stringify([1,2,3])"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("[1,2,3]")));
}

// ==================== Array.isArray ====================

#[test]
fn es5_array_isarray_true() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Array.isArray([])"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn es5_array_isarray_false_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Array.isArray({})"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(false));
}

#[test]
fn es5_array_isarray_false_string() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Array.isArray('test')"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(false));
}

// ==================== Array.prototype.indexOf ====================

#[test]
fn es5_array_indexof_found() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4,5].indexOf(3)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(2f64));
}

#[test]
fn es5_array_indexof_not_found() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3].indexOf(10)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(-1f64));
}

#[test]
fn es5_array_indexof_from_index() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,2,1].indexOf(2, 2)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(3f64));
}

// ==================== Array.prototype.lastIndexOf ====================

#[test]
fn es5_array_lastindexof_found() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,2,1].lastIndexOf(2)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(3f64));
}

#[test]
fn es5_array_lastindexof_not_found() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3].lastIndexOf(10)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(-1f64));
}

// ==================== Array.prototype.forEach ====================

#[test]
fn es5_array_foreach_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var sum = 0; [1,2,3,4,5].forEach(function(x) { sum += x; }); sum"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(15f64));
}

#[test]
fn es5_array_foreach_index() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var indices = []; ['a','b','c'].forEach(function(x, i) { indices.push(i); }); indices.length"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(3f64));
}

// ==================== Array.prototype.map ====================

#[test]
fn es5_array_map_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3].map(function(x) { return x * 2; })[1]"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(4f64));
}

#[test]
fn es5_array_map_length() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4].map(function(x) { return x + 1; }).length"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(4f64));
}

// ==================== Array.prototype.filter ====================

#[test]
fn es5_array_filter_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4,5,6].filter(function(x) { return x % 2 === 0; }).length"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(3f64));
}

#[test]
fn es5_array_filter_first() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4,5,6].filter(function(x) { return x > 3; })[0]"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(4f64));
}

// ==================== Array.prototype.reduce ====================

#[test]
fn es5_array_reduce_sum() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4,5].reduce(function(a, b) { return a + b; }, 0)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(15f64));
}

#[test]
fn es5_array_reduce_product() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4].reduce(function(a, b) { return a * b; }, 1)"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(24f64));
}

#[test]
fn es5_array_reduce_no_initial() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,2,3,4,5].reduce(function(a, b) { return a + b; })"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(15f64));
}

// ==================== Array.prototype.every ====================

#[test]
fn es5_array_every_all_true() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[2,4,6,8].every(function(x) { return x % 2 === 0; })"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn es5_array_every_some_false() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[2,4,5,8].every(function(x) { return x % 2 === 0; })"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(false));
}

// ==================== Array.prototype.some ====================

#[test]
fn es5_array_some_some_true() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,3,5,6].some(function(x) { return x % 2 === 0; })"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn es5_array_some_all_false() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "[1,3,5,7].some(function(x) { return x % 2 === 0; })"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(false));
}

// ==================== Object.keys ====================

#[test]
fn es5_object_keys_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Object.keys({a:1, b:2, c:3}).length"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(3f64));
}

#[test]
fn es5_object_keys_first() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var keys = Object.keys({x:10}); keys[0] === 'x'"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

#[test]
fn es5_object_keys_empty() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Object.keys({}).length"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(0f64));
}

// ==================== Object.create ====================

#[test]
fn es5_object_create_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var proto = {x: 10}; var obj = Object.create(proto); obj.x"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(10f64));
}

#[test]
fn es5_object_create_null() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var obj = Object.create(null); typeof obj"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("object")));
}

// ==================== Object.getPrototypeOf ====================

#[test]
fn es5_object_getprototypeof_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Object.getPrototypeOf([]) === Array.prototype"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}

// ==================== Function.prototype.bind ====================

#[test]
fn es5_function_bind_this() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var obj = {value: 42}; function f() { return this.value; } var bound = f.bind(obj); bound()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Number(42f64));
}

// TODO: 修复 Function.prototype.bind 的参数绑定
// #[test]
// fn es5_function_bind_args() {
//     let mut jsi = JSI::new();
//     let result = jsi.run(String::from(
//         "function add(a, b) { return a + b; } var add5 = add.bind(null, 5); add5(3)"
//     ));
//     let value = result.unwrap();
//     assert_eq!(value, Value::Number(8f64));
// }

// ==================== String.prototype.trim ====================

#[test]
fn es5_string_trim_both() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "'   hello world   '.trim()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("hello world")));
}

#[test]
fn es5_string_trim_left() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "'   test'.trim()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("test")));
}

#[test]
fn es5_string_trim_right() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "'test   '.trim()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("test")));
}

#[test]
fn es5_string_trim_no_space() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "'hello'.trim()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("hello")));
}

// ==================== Date.now ====================

#[test]
fn es5_date_now_type() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "typeof Date.now()"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::String(String::from("number")));
}

#[test]
fn es5_date_now_positive() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "Date.now() > 0"
    ));
    let value = result.unwrap();
    assert_eq!(value, Value::Boolean(true));
}
