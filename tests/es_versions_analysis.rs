use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::Path,
};
use yaml_rust::YamlLoader;

#[derive(Debug, Default)]
struct ESVersionStats {
    total: usize,
    es1: usize, // ES1 (ECMAScript 1997) - 基础语法
    es3: usize, // ES3 (ECMAScript 1999) - 正则、异常处理
    es5: usize, // ES5 (ECMAScript 5.1) - 严格模式、JSON、数组方法
    es6: usize, // ES6 (ES2015) - Promise、箭头函数、类、let/const
    es2016: usize,
    es2017: usize,
    es2018: usize,
    es2019: usize,
    es2020: usize,
    es2021: usize,
    es2022: usize,
    es2023: usize,
    es2024: usize,
    unknown: usize,
}

#[derive(Debug, Default)]
struct FeatureStats {
    total: usize,
    features: HashMap<String, usize>,
}

fn parse_test_file(path: &Path) -> (Option<String>, Vec<String>) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, Vec::new()),
    };

    let mut code = String::new();
    if file.read_to_string(&mut code).is_err() {
        return (None, Vec::new());
    }

    let start = code.find("/*---");
    let end = code.find("---*/");

    let (es_version, features) = match (start, end) {
        (Some(start), Some(end)) => {
            let config = &code[start + 5..end];
            let docs = YamlLoader::load_from_str(config);

            let mut es_version = None;
            let mut features: Vec<String> = Vec::new();

            if let Ok(docs) = docs {
                // 检测 ES 版本标记
                if let Some(es5id) = docs[0]["es5id"].as_str() {
                    es_version = Some("ES5".to_string());
                } else if let Some(es6id) = docs[0]["es6id"].as_str() {
                    es_version = Some("ES6".to_string());
                } else if let Some(esid) = docs[0]["esid"].as_str() {
                    // 解析 esid 前缀确定版本
                    if esid.starts_with("sec-") {
                        // 这是 ES2015+ 的格式，需要进一步分析
                        es_version = Some("ES6+".to_string());
                    } else if esid == "pending" {
                        es_version = Some("Pending".to_string());
                    } else {
                        es_version = Some("Modern".to_string());
                    }
                }

                // 检测 features 字段
                if let yaml_rust::Yaml::Array(arr) = &docs[0]["features"] {
                    for feature in arr {
                        if let Some(name) = feature.as_str() {
                            features.push(name.to_string());
                        }
                    }
                }
            }

            (es_version, features)
        }
        _ => (None, Vec::new()),
    };

    // 基于文件名/路径的启发式检测
    let path_str = path.to_string_lossy();
    let detected_version = if path_str.contains("S7.") || path_str.contains("S8.") || path_str.contains("S9.")
        || path_str.contains("S10.") || path_str.contains("S11.") || path_str.contains("S12.")
        || path_str.contains("S13.") || path_str.contains("S14.") || path_str.contains("S15.")
    {
        // Sputnik 测试命名通常是 ES1/ES3 时代的
        Some("ES1/ES3".to_string())
    } else {
        None
    };

    (es_version.or(detected_version), features)
}

fn categorize_by_path(path: &Path) -> Option<&'static str> {
    let path_str = path.to_string_lossy();

    // ES5 特性目录
    if path_str.contains("strict-mode") || path_str.contains("isArray")
        || path_str.contains("getOwnPropertyDescriptor") || path_str.contains("defineProperty")
        || path_str.contains("getOwnPropertyNames") || path_str.contains("Object.create")
        || path_str.contains("keys") || path_str.contains("JSON")
        || path_str.contains("forEach") || path_str.contains("map") || path_str.contains("filter")
        || path_str.contains("every") || path_str.contains("some") || path_str.contains("reduce")
        || path_str.contains("indexOf") || path_str.contains("lastIndexOf")
        || path_str.contains("trim") || path_str.contains("bind")
        || path_str.contains("Date.now") || path_str.contains("Date.toISOString")
    {
        return Some("ES5");
    }

    // ES6 (ES2015) 特性
    if path_str.contains("arrow-function") || path_str.contains("let")
        || path_str.contains("const") || path_str.contains("class")
        || path_str.contains("generator") || path_str.contains("yield")
        || path_str.contains("destructuring") || path_str.contains("spread")
        || path_str.contains("rest-parameters") || path_str.contains("default-parameters")
        || path_str.contains("template") || path_str.contains("Promise")
        || path_str.contains("Map") || path_str.contains("Set")
        || path_str.contains("Symbol") || path_str.contains("Proxy")
        || path_str.contains("Reflect") || path_str.contains("for-of")
        || path_str.contains("module") || path_str.contains("import")
    {
        return Some("ES6");
    }

    // ES2016
    if path_str.contains("Array.prototype.includes") || path_str.contains("**") {
        return Some("ES2016");
    }

    // ES2017
    if path_str.contains("async") || path_str.contains("await")
        || path_str.contains("padStart") || path_str.contains("padEnd")
        || path_str.contains("Object.values") || path_str.contains("Object.entries")
        || path_str.contains("Object.getOwnPropertyDescriptors")
    {
        return Some("ES2017");
    }

    // ES2018
    if path_str.contains("rest-spread") || path_str.contains("Promise.prototype.finally")
        || path_str.contains("async-iteration") || path_str.contains("asynchronous-iteration")
    {
        return Some("ES2018");
    }

    // ES2019
    if path_str.contains("flat") || path_str.contains("flatMap")
        || path_str.contains("Object.fromEntries")
        || path_str.contains("trimStart") || path_str.contains("trimEnd")
    {
        return Some("ES2019");
    }

    // ES2020
    if path_str.contains("BigInt") || path_str.contains("globalThis")
        || path_str.contains("Promise.allSettled") || path_str.contains("optional-chaining")
        || path_str.contains("coalesce-expression") || path_str.contains("matchAll")
        || path_str.contains("dynamic-import")
    {
        return Some("ES2020");
    }

    // ES2021
    if path_str.contains("logical-assignment") || path_str.contains("Promise.any")
        || path_str.contains("replaceAll") || path_str.contains("WeakRef")
        || path_str.contains("FinalizationRegistry")
    {
        return Some("ES2021");
    }

    // ES2022
    if path_str.contains("class-fields") || path_str.contains("private-methods")
        || path_str.contains("static-block") || path_str.contains("at-method")
        || path_str.contains("Object.hasOwn")
        || path_str.contains("error-cause") || path_str.contains("cause")
    {
        return Some("ES2022");
    }

    // ES2023
    if path_str.contains("change-array-by-copy") || path_str.contains("toReversed")
        || path_str.contains("toSorted") || path_str.contains("toSpliced")
        || path_str.contains("with") || path_str.contains("findLast")
        || path_str.contains("findLastIndex") || path_str.contains("groupBy")
    {
        return Some("ES2023");
    }

    None
}

fn collect_tests(path: &Path, stats: &mut ESVersionStats, feature_stats: &mut FeatureStats) {
    if !path.exists() {
        return;
    }

    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return,
    };

    if metadata.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            collect_tests(&entry.path(), stats, feature_stats);
        }
    } else if path.to_string_lossy().ends_with(".js") {
        stats.total += 1;

        let (version_tag, features) = parse_test_file(path);

        // 添加到特性统计
        for feature in features {
            *feature_stats.features.entry(feature).or_insert(0) += 1;
            feature_stats.total += 1;
        }

        // 基于路径分类
        let path_category = categorize_by_path(path);

        // 确定最终分类
        match (version_tag.as_deref(), path_category) {
            (Some("ES5"), _) | (_, Some("ES5")) => stats.es5 += 1,
            (Some("ES6"), _) | (_, Some("ES6")) => stats.es6 += 1,
            (_, Some("ES2016")) => stats.es2016 += 1,
            (_, Some("ES2017")) => stats.es2017 += 1,
            (_, Some("ES2018")) => stats.es2018 += 1,
            (_, Some("ES2019")) => stats.es2019 += 1,
            (_, Some("ES2020")) => stats.es2020 += 1,
            (_, Some("ES2021")) => stats.es2021 += 1,
            (_, Some("ES2022")) => stats.es2022 += 1,
            (_, Some("ES2023")) => stats.es2023 += 1,
            (Some("ES1/ES3"), _) => stats.es3 += 1, // 暂时归入 ES3
            (Some("Modern"), _) => stats.es6 += 1, // 现代特性归入 ES6+
            (Some("ES6+"), _) => stats.es6 += 1,
            _ => stats.unknown += 1,
        }
    }
}

fn main() {
    println!("=== ECMAScript Version Test Analysis ===\n");

    let test262_path = Path::new("./test262/test");

    let mut stats = ESVersionStats::default();
    let mut feature_stats = FeatureStats::default();

    println!("Collecting test files...");
    collect_tests(test262_path, &mut stats, &mut feature_stats);

    println!("\n=== Test Count by ES Version ===");
    println!("Total tests: {}", stats.total);
    println!("ES1/ES3:      {} (基础语法、正则、异常处理)", stats.es3);
    println!("ES5:          {} (严格模式、JSON、数组方法)", stats.es5);
    println!("ES6 (ES2015): {} (箭头函数、类、Promise、let/const)", stats.es6);
    println!("ES2016:       {} (Array.includes、指数运算符)", stats.es2016);
    println!("ES2017:       {} (async/await、padStart/End)", stats.es2017);
    println!("ES2018:       {} (对象展开、Promise.finally)", stats.es2018);
    println!("ES2019:       {} (flat/flatMap、fromEntries)", stats.es2019);
    println!("ES2020:       {} (BigInt、globalThis、可选链)", stats.es2020);
    println!("ES2021:       {} (逻辑赋值、Promise.any)", stats.es2021);
    println!("ES2022:       {} (类字段、at方法、error cause)", stats.es2022);
    println!("ES2023:       {} (数组复制方法、findLast)", stats.es2023);
    println!("ES2024:       {}", stats.es2024);
    println!("Unknown:      {}", stats.unknown);

    // 计算百分比
    let classified = stats.es3 + stats.es5 + stats.es6 + stats.es2016 + stats.es2017
        + stats.es2018 + stats.es2019 + stats.es2020 + stats.es2021 + stats.es2022 + stats.es2023;

    println!("\n=== Classification Rate ===");
    println!("Classified:   {} / {} ({:.1}%)", classified, stats.total,
             (classified as f64 / stats.total as f64) * 100.0);

    println!("\n=== Top 30 Features by Test Count ===");
    let mut feature_vec: Vec<_> = feature_stats.features.iter().collect();
    feature_vec.sort_by(|a, b| b.1.cmp(a.1));

    for (i, (feature, count)) in feature_vec.iter().take(30).enumerate() {
        println!("{:2}. {:<35} {:>5} tests", i + 1, format!("\"{}\"", feature), count);
    }

    println!("\n=== Key Features Summary ===");
    let key_es5_features = ["JSON", "strict-mode", "array-includes", "string-trimming"];
    let key_es6_features = ["arrow-function", "let", "const", "class", "Promise", "Map", "Set",
                            "Symbol", "generators", "destructuring-binding", "for-of", "default-parameters"];

    println!("\nES5 Features:");
    for &feature in &key_es5_features {
        let count = feature_stats.features.get(feature).copied().unwrap_or(0);
        println!("  {:<20} {:>5} tests", feature, count);
    }

    println!("\nES6 (ES2015) Features:");
    for &feature in &key_es6_features {
        let count = feature_stats.features.get(feature).copied().unwrap_or(0);
        println!("  {:<25} {:>5} tests", feature, count);
    }

    println!("\n=== Note ===");
    println!("This classification is based on:");
    println!("1. Explicit metadata tags (es5id, es6id, features)");
    println!("2. Directory/file naming heuristics");
    println!("Some tests may overlap between categories.");
}
