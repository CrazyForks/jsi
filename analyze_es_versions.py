#!/usr/bin/env python3
"""
Analyze test262 tests by ECMAScript version
"""
import os
import re
import json
from collections import defaultdict
from pathlib import Path

TEST262_PATH = Path("./test262/test")

# ES 版本到特性的映射
ES_FEATURES = {
    "ES1": [
        # ES1 (1997) - 基础语法
        "var", "function", "if", "else", "for", "while", "do", "switch",
        "Array", "String", "Boolean", "Number", "Date", "Math", "Object",
        "RegExp", "Error", "EvalError", "RangeError", "ReferenceError",
        "SyntaxError", "TypeError", "try", "catch", "throw", "finally"
    ],
    "ES3": [
        # ES3 (1999) - 正则、异常处理
        "regexp", "try-catch", "do-while",
    ],
    "ES5": [
        # ES5 (2009)
        "JSON", "strict-mode", "strict", "Object.create",
        "Object.defineProperty", "Object.defineProperties",
        "Object.getPrototypeOf", "Object.keys", "Object.getOwnPropertyNames",
        "Object.getOwnPropertyDescriptor", "Object.preventExtensions",
        "Object.isExtensible", "Object.seal", "Object.isSealed",
        "Object.freeze", "Object.isFrozen",
        "Array.isArray", "Array.prototype.every",
        "Array.prototype.filter", "Array.prototype.forEach",
        "Array.prototype.indexOf", "Array.prototype.lastIndexOf",
        "Array.prototype.map", "Array.prototype.reduce",
        "Array.prototype.reduceRight", "Array.prototype.some",
        "Function.prototype.bind", "String.prototype.trim",
        "Date.now", "Date.prototype.toISOString",
    ],
    "ES6": [
        # ES6 (ES2015)
        "arrow-function", "let", "const", "class", "class-fields",
        "generators", "yield", "destructuring", "spread", "spread-syntax",
        "rest-parameters", "default-parameters", "template",
        "Promise", "Map", "Set", "WeakMap", "WeakSet", "Symbol",
        "Proxy", "Reflect", "for-of", "module", "import", "export",
        "Symbol.iterator", "Symbol.match", "Symbol.replace",
        "Symbol.search", "Symbol.species", "Symbol.split",
        "Symbol.toPrimitive", "Symbol.toStringTag",
        "new.target", "Object.assign", "Array.from",
        "Array.prototype.fill", "Array.prototype.find",
        "Array.prototype.findIndex",
        "String.fromCodePoint", "String.prototype.codePointAt",
        "String.prototype.includes", "String.prototype.startsWith",
        "String.prototype.endsWith", "String.prototype.repeat",
        "Number.isFinite", "Number.isInteger", "Number.isSafeInteger",
        "Number.isNaN", "Number.EPSILON", "Number.MIN_SAFE_INTEGER",
        "Number.MAX_SAFE_INTEGER", "Math.clz32", "Math.imul",
        "Math.sign", "Math.log10", "Math.log2", "Math.log1p",
        "Math.expm1", "Math.cosh", "Math.sinh", "Math.tanh",
        "Math.acosh", "Math.asinh", "Math.atanh", "Math.hypot",
        "Math.trunc", "Math.fround", "Math.cbrt",
    ],
    "ES2016": [
        # ES2016
        "Array.prototype.includes", "Array.includes", "exponentiation",
    ],
    "ES2017": [
        # ES2017
        "async", "async-functions", "await",
        "Object.values", "Object.entries", "Object.getOwnPropertyDescriptors",
        "padStart", "padEnd", "Atomics", "SharedArrayBuffer",
    ],
    "ES2018": [
        # ES2018
        "Promise.prototype.finally", "finally", "async-iteration",
        "asynchronous-iteration", "object-rest", "object-spread",
        "regexp-lookbehind", "regexp-named-groups", "regexp-unicode-property-escapes",
        "regexp-dotall",
    ],
    "ES2019": [
        # ES2019
        "Array.prototype.flat", "Array.prototype.flatMap", "flatMap",
        "Object.fromEntries", "String.prototype.trimStart", "trimStart",
        "String.prototype.trimEnd", "trimEnd",
        "Symbol.prototype.description",
        "catch-binding", "optional-catch-binding",
        "try {} catch {}",
    ],
    "ES2020": [
        # ES2020
        "BigInt", "globalThis", "Promise.allSettled",
        "optional-chaining", "nullish-coalescing", "coalesce-expression",
        "String.prototype.matchAll", "matchAll", "dynamic-import",
        "import-assertions",
    ],
    "ES2021": [
        # ES2021
        "logical-assignment-operators", "Promise.any",
        "String.prototype.replaceAll", "replaceAll",
        "WeakRef", "FinalizationRegistry",
    ],
    "ES2022": [
        # ES2022
        "class-fields-private", "class-methods-private", "class-static-block",
        "class-static-fields-private", "class-static-methods-private",
        "Error.prototype.cause", "error-cause",
        "Object.hasOwn", "at", "String.prototype.at", "Array.prototype.at",
        "regexp-match-indices",
    ],
    "ES2023": [
        # ES2023
        "change-array-by-copy", "toReversed", "toSorted", "toSpliced",
        "with", "Array.prototype.with", "findLast", "findLastIndex",
        "Array.prototype.findLast", "array-grouping", "groupBy",
        "Symbol.dispose", "Symbol.asyncDispose",
        "explicit-resource-management",
    ],
}

# 按版本顺序排列
ES_VERSIONS_ORDER = [
    "ES2023", "ES2022", "ES2021", "ES2020", "ES2019", "ES2018",
    "ES2017", "ES2016", "ES6", "ES5", "ES3", "ES1"
]

def parse_test_file(filepath):
    """Parse test file metadata"""
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
    except:
        return None, []

    # 提取 YAML 元数据
    start = content.find("/*---")
    end = content.find("---*/")
    if start == -1 or end == -1:
        return None, []

    yaml_content = content[start + 5:end]

    # 提取 ES 版本
    es_version = None
    if re.search(r'\bes5id\s*:', yaml_content):
        es_version = "ES5"
    elif re.search(r'\bes6id\s*:', yaml_content):
        es_version = "ES6"

    # 提取 features
    features = []
    feature_match = re.search(r'features\s*:\s*\[([^\]]+)\]', yaml_content)
    if feature_match:
        features_str = feature_match.group(1)
        features = [f.strip().strip('"\'') for f in features_str.split(',')]

    return es_version, features

def categorize_by_path(filepath):
    """Categorize test by file path"""
    path_str = str(filepath)

    # 检查特定版本的目录/文件名
    for version in reversed(ES_VERSIONS_ORDER):
        for feature in ES_FEATURES[version]:
            feature_lower = feature.lower()
            if feature_lower in path_str.lower():
                return version

    # 检查 ES1/ES3 时代的 Sputnik 测试
    if re.search(r'/S\d+\.', path_str) or re.search(r'/S\d+_', path_str):
        return "ES3"  # ES1/ES3 时代的测试

    return None

def count_tests_by_version():
    """Count tests by ES version"""
    version_counts = defaultdict(int)
    feature_counts = defaultdict(int)
    all_features = set()

    total = 0

    for root, dirs, files in os.walk(TEST262_PATH):
        for filename in files:
            if not filename.endswith('.js'):
                continue

            filepath = Path(root) / filename
            total += 1

            es_version, features = parse_test_file(filepath)

            # 添加到特性统计
            for feature in features:
                feature_counts[feature] += 1
                all_features.add(feature)

            # 按路径分类
            path_category = categorize_by_path(filepath)

            # 确定最终版本（优先使用元数据标记）
            final_version = es_version or path_category or "Unknown"
            version_counts[final_version] += 1

    return version_counts, feature_counts, total

def print_statistics():
    """Print statistics"""
    print("=" * 60)
    print("ECMAScript Version Test Analysis")
    print("=" * 60)

    version_counts, feature_counts, total = count_tests_by_version()

    print(f"\nTotal tests: {total}")
    print("\n" + "-" * 60)
    print("Test Count by ES Version:")
    print("-" * 60)

    classified = 0
    for version in ES_VERSIONS_ORDER:
        count = version_counts.get(version, 0)
        classified += count
        percentage = (count / total * 100) if total > 0 else 0
        print(f"{version:8} {count:>6}  ({percentage:>5.1f}%)")

    unknown = version_counts.get("Unknown", 0)
    percentage = (unknown / total * 100) if total > 0 else 0
    print(f"{'Unknown':8} {unknown:>6}  ({percentage:>5.1f}%)")

    print("-" * 60)
    classified_percent = (classified / total * 100) if total > 0 else 0
    print(f"{'Classified':8} {classified:>6}  ({classified_percent:>5.1f}%)")

    print("\n" + "-" * 60)
    print("Top 30 Features by Test Count:")
    print("-" * 60)

    sorted_features = sorted(feature_counts.items(), key=lambda x: x[1], reverse=True)
    for i, (feature, count) in enumerate(sorted_features[:30], 1):
        print(f"{i:2}. {feature:35} {count:>5} tests")

    print("\n" + "-" * 60)
    print("Key Features by ES Version:")
    print("-" * 60)

    # ES5 关键特性
    print("\nES5 Features:")
    es5_key_features = ["JSON", "strict-mode", "arrow-function",
                        "Array.prototype.includes", "String.prototype.trim"]
    for feature in es5_key_features:
        if feature in feature_counts:
            print(f"  {feature:25} {feature_counts[feature]:>5} tests")

    # ES6 关键特性
    print("\nES6 (ES2015) Features:")
    es6_key_features = ["arrow-function", "let", "const", "class",
                        "generators", "Promise", "Map", "Set", "Symbol",
                        "Proxy", "Reflect", "destructuring-binding", "for-of"]
    for feature in es6_key_features:
        if feature in feature_counts:
            print(f"  {feature:25} {feature_counts[feature]:>5} tests")

    # 现代特性
    print("\nModern ES Features (ES2016+):")
    modern_features = ["async-functions", "BigInt", "globalThis",
                       "optional-chaining", "coalesce-expression",
                       "Promise.allSettled", "Promise.any",
                       "class-fields-private", "change-array-by-copy",
                       "explicit-resource-management"]
    for feature in modern_features:
        if feature in feature_counts:
            print(f"  {feature:30} {feature_counts[feature]:>5} tests")

    # 保存详细统计到 JSON
    result = {
        "total": total,
        "version_counts": dict(version_counts),
        "top_features": dict(sorted_features[:50]),
    }

    with open("es_version_stats.json", "w") as f:
        json.dump(result, f, indent=2)

    print(f"\nDetailed statistics saved to es_version_stats.json")

if __name__ == "__main__":
    print_statistics()
