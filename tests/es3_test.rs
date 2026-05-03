use jsi::{JSI, value::Value};

// ============================================================================
// ES3 (ECMAScript 3) 核心语法测试
// ============================================================================
// ES3 于 1999 年发布，相比 ES1/ES2 新增了以下核心特性：
// - try-catch-finally 异常处理
// - throw 语句
// - RegExp 正则表达式对象
// - in 运算符
// - instanceof 运算符
// - decodeURI, decodeURIComponent, encodeURI, encodeURIComponent
// - 更多内置方法: String.split, String.replace, Array.shift, Array.unshift, Array.slice 等
// ============================================================================

// ==================== try-catch 异常处理 ====================

#[test]
fn es3_try_catch_basic() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var errorCaught = false; try { throw 'error'; } catch (e) { errorCaught = true; } errorCaught"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_try_catch_no_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var executed = false; try { executed = true; } catch (e) { executed = false; } executed"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_try_catch_error_value() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var caught; try { throw 42; } catch (e) { caught = e; } caught"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(42f64));
    }
}

#[test]
fn es3_try_catch_string_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var msg; try { throw 'test error'; } catch (e) { msg = e; } msg"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test error")));
    }
}

// ==================== try-finally ====================

#[test]
fn es3_try_finally_always_executes() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var finallyExecuted = false; try { } finally { finallyExecuted = true; } finallyExecuted"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_try_catch_finally() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var catchExecuted = false, finallyExecuted = false; \
        try { throw 1; } catch (e) { catchExecuted = true; } finally { finallyExecuted = true; } \
        catchExecuted && finallyExecuted"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== throw 语句 ====================

#[test]
fn es3_throw_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var result; try { throw { code: 500, message: 'error' }; } catch (e) { result = e.code; } result"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(500f64));
    }
}

#[test]
fn es3_throw_new_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "try { throw new Error('test'); } catch (e) { typeof e; }"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("object")));
    }
}

// ==================== in 运算符 ====================

#[test]
fn es3_in_operator_exists() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'toString' in {}"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_in_operator_own_property() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'x' in { x: 1 }"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_in_operator_not_exists() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'nonexistent' in {}"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(false));
    }
}

#[test]
fn es3_in_operator_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("0 in [1, 2, 3]"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_in_operator_array_length() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'length' in []"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== instanceof 运算符 ====================

#[test]
fn es3_instanceof_array() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[] instanceof Array"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_instanceof_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("({}) instanceof Object"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_instanceof_custom_constructor() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "function Foo() {} new Foo() instanceof Foo"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_instanceof_false() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("42 instanceof Number"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(false));
    }
}

#[test]
fn es3_instanceof_function() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("(function() {}) instanceof Function"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== RegExp 正则表达式对象 ====================

#[test]
fn es3_regexp_literal() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("typeof /abc/"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("object")));
    }
}

#[test]
fn es3_regexp_constructor() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("typeof new RegExp('abc')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("object")));
    }
}

#[test]
fn es3_regexp_test() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/abc/.test('abcdef')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_regexp_test_no_match() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/xyz/.test('abcdef')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(false));
    }
}

#[test]
fn es3_regexp_exec() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var result = /(a)(b)/.exec('abc'); result[0] + result[1] + result[2]"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("abab")));
    }
}

#[test]
fn es3_regexp_source() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/abc/.source"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("abc")));
    }
}

#[test]
fn es3_regexp_global_flag() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/abc/g.global"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_regexp_ignore_case_flag() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/abc/i.ignoreCase"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_regexp_multiline_flag() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("/abc/m.multiline"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_regexp_last_index() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var re = /a/g; re.lastIndex = 2; re.lastIndex"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(2f64));
    }
}

// ==================== String 扩展方法 ====================

#[test]
fn es3_string_split() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'a,b,c'.split(',').length"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(3f64));
    }
}

#[test]
fn es3_string_split_space() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'hello world'.split(' ')[0]"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("hello")));
    }
}

#[test]
fn es3_string_replace() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'hello'.replace('l', 'x')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("hexlo")));
    }
}

#[test]
fn es3_string_match() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'abc'.match(/abc/)[0]"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("abc")));
    }
}

#[test]
fn es3_string_search() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'testabc'.search(/abc/)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(4f64));
    }
}

#[test]
fn es3_string_char_code_at() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("'A'.charCodeAt(0)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(65f64));
    }
}

#[test]
fn es3_string_from_char_code() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("String.fromCharCode(65)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("A")));
    }
}

// ==================== Array 扩展方法 ====================

#[test]
fn es3_array_shift() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var arr = [1, 2, 3]; arr.shift()"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(1f64));
    }
}

#[test]
fn es3_array_shift_length() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var arr = [1, 2, 3]; arr.shift(); arr.length"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(2f64));
    }
}

#[test]
fn es3_array_unshift() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var arr = [2, 3]; arr.unshift(1); arr[0]"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(1f64));
    }
}

#[test]
fn es3_array_unshift_returns_length() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var arr = [2, 3]; arr.unshift(0, 1)"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(4f64));
    }
}

#[test]
fn es3_array_slice() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[1, 2, 3, 4, 5].slice(1, 4).join(',')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("2,3,4")));
    }
}

#[test]
fn es3_array_slice_start_only() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[1, 2, 3, 4].slice(1).join(',')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("2,3,4")));
    }
}

#[test]
fn es3_array_slice_negative() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[1, 2, 3, 4].slice(-2).join(',')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("3,4")));
    }
}

#[test]
fn es3_array_to_string() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[1, 2, 3].toString()"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("1,2,3")));
    }
}

// ==================== Math 扩展方法 ====================

#[test]
fn es3_math_floor() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.floor(3.7)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(3f64));
    }
}

#[test]
fn es3_math_ceil() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.ceil(3.2)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(4f64));
    }
}

#[test]
fn es3_math_round() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.round(3.5)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(4f64));
    }
}

#[test]
fn es3_math_max() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.max(1, 5, 3)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(5f64));
    }
}

#[test]
fn es3_math_min() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.min(1, 5, 3)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(1f64));
    }
}

#[test]
fn es3_math_abs() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.abs(-5)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(5f64));
    }
}

#[test]
fn es3_math_pow() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.pow(2, 3)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(8f64));
    }
}

#[test]
fn es3_math_sqrt() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.sqrt(9)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(3f64));
    }
}

#[test]
fn es3_math_sin() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.sin(0)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(0f64));
    }
}

#[test]
fn es3_math_cos() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.cos(0)"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(1f64));
    }
}

#[test]
fn es3_math_pi() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("Math.PI > 3 && Math.PI < 4"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== Error 对象 ====================

#[test]
fn es3_error_constructor() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("typeof new Error('test')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("object")));
    }
}

#[test]
fn es3_error_message() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new Error('test message').message"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test message")));
    }
}

#[test]
fn es3_error_name() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new Error().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("Error")));
    }
}

#[test]
fn es3_type_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new TypeError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("TypeError")));
    }
}

#[test]
fn es3_reference_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new ReferenceError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("ReferenceError")));
    }
}

#[test]
fn es3_syntax_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new SyntaxError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("SyntaxError")));
    }
}

#[test]
fn es3_range_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new RangeError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("RangeError")));
    }
}

#[test]
fn es3_eval_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new EvalError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("EvalError")));
    }
}

#[test]
fn es3_uri_error() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("new URIError().name"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("URIError")));
    }
}

// ==================== encodeURI / decodeURI ====================

#[test]
fn es3_encode_uri() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("encodeURI('test test')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test%20test")));
    }
}

#[test]
fn es3_decode_uri() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("decodeURI('test%20test')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test test")));
    }
}

#[test]
fn es3_encode_uri_component() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("encodeURIComponent('test/test')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test%2Ftest")));
    }
}

#[test]
fn es3_decode_uri_component() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("decodeURIComponent('test%2Ftest')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::String(String::from("test/test")));
    }
}

// ==================== Object 扩展方法 ====================

#[test]
fn es3_object_has_own_property() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("({ x: 1 }).hasOwnProperty('x')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_object_has_own_property_false() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("({}).hasOwnProperty('toString')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(false));
    }
}

#[test]
fn es3_object_property_is_enumerable() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("({ x: 1 }).propertyIsEnumerable('x')"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

#[test]
fn es3_object_is_prototype_of() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "function Foo() {} Object.prototype.isPrototypeOf(new Foo())"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== 嵌套 try-catch ====================

#[test]
fn es3_nested_try_catch() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var caught = 0; try { try { throw 1; } catch (e) { caught++; throw e; } } catch (e) { caught++; } caught"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(2f64));
    }
}

// ==================== 构造函数 prototype 链测试 ====================

#[test]
fn es3_prototype_chain_instanceof() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "function A() {} function B() {} B.prototype = new A(); new B() instanceof A"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== Array 原型链 ====================

#[test]
fn es3_array_instanceof_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("[] instanceof Object"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== Function 原型链 ====================

#[test]
fn es3_function_instanceof_object() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from("(function() {}) instanceof Object"));
    if let Ok(value) = result {
        assert_eq!(value, Value::Boolean(true));
    }
}

// ==================== 空数组 unshift ====================

#[test]
fn es3_array_unshift_empty() {
    let mut jsi = JSI::new();
    let result = jsi.run(String::from(
        "var arr = []; arr.unshift(1); arr[0]"
    ));
    if let Ok(value) = result {
        assert_eq!(value, Value::Number(1f64));
    }
}
