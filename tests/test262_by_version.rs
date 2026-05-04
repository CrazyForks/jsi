use std::{
    path::{Path, PathBuf},
    env, fs::{self, File},
    io::{Write, Read},
    panic, thread,
    collections::HashMap,
};
use jsi::JSI;
use serde::{Serialize, Deserialize};
use yaml_rust::{YamlLoader, Yaml};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum ESVersion {
    ES1,
    ES3,
    ES5,
    ES6,
    ES2016,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2023,
    Unknown,
}

impl ESVersion {
    fn to_str(&self) -> &'static str {
        match self {
            ESVersion::ES1 => "ES1",
            ESVersion::ES3 => "ES3",
            ESVersion::ES5 => "ES5",
            ESVersion::ES6 => "ES6",
            ESVersion::ES2016 => "ES2016",
            ESVersion::ES2017 => "ES2017",
            ESVersion::ES2018 => "ES2018",
            ESVersion::ES2019 => "ES2019",
            ESVersion::ES2020 => "ES2020",
            ESVersion::ES2021 => "ES2021",
            ESVersion::ES2022 => "ES2022",
            ESVersion::ES2023 => "ES2023",
            ESVersion::Unknown => "Unknown",
        }
    }
}

#[derive(Clone)]
struct VersionResult {
    pub version: ESVersion,
    pub cases: usize,
    pub passed: usize,
    pub files: Vec<(String, bool)>,
}

impl VersionResult {
    fn new(version: ESVersion) -> Self {
        VersionResult {
            version,
            cases: 0,
            passed: 0,
            files: Vec::new(),
        }
    }

    fn add_result(&mut self, name: String, passed: bool) {
        self.cases += 1;
        if passed {
            self.passed += 1;
        }
        self.files.push((name, passed));
    }
}

#[derive(Clone)]
struct Test262File {
    pub name: String,
    pub path: String,
    pub code: String,
    pub no_strict: bool,
    pub negative: bool,
    pub negative_type: String,
    pub version: ESVersion,
}

impl Test262File {
    pub fn new(name: String, path: String) -> Test262File {
        let mut file = File::open(&path).unwrap();
        let mut code = String::new();
        file.read_to_string(&mut code).unwrap();
        let (no_strict, negative, negative_type, version) = Test262File::parse(&code, &path);
        return Test262File {
            name,
            path,
            code,
            no_strict,
            negative,
            negative_type,
            version,
        }
    }

    fn detect_version_by_meta(code: &str) -> Option<ESVersion> {
        let start = code.find("/*---");
        let end = code.find("---*/");
        if start.is_none() || end.is_none() {
            return None;
        }

        let config = &code[start.unwrap() + 5..end.unwrap()];
        let docs = YamlLoader::load_from_str(config);
        if let Ok(docs) = docs {
            // 检测 es5id/es6id
            if let Some(_) = docs[0]["es5id"].as_str() {
                return Some(ESVersion::ES5);
            }
            if let Some(_) = docs[0]["es6id"].as_str() {
                return Some(ESVersion::ES6);
            }

            // 检测 features 字段
            if let Yaml::Array(arr) = &docs[0]["features"] {
                for feature in arr {
                    if let Some(name) = feature.as_str() {
                        match name {
                            // ES2016+ features
                            "arrow-function" => return Some(ESVersion::ES6),
                            "destructuring-binding" => return Some(ESVersion::ES6),
                            "default-parameters" => return Some(ESVersion::ES6),
                            "class" => return Some(ESVersion::ES6),
                            "generators" => return Some(ESVersion::ES6),
                            "Symbol" => return Some(ESVersion::ES6),
                            "Symbol.iterator" => return Some(ESVersion::ES6),
                            "for-of" => return Some(ESVersion::ES6),
                            "let" => return Some(ESVersion::ES6),
                            "const" => return Some(ESVersion::ES6),
                            "Promise" => return Some(ESVersion::ES6),
                            "Map" => return Some(ESVersion::ES6),
                            "Set" => return Some(ESVersion::ES6),
                            "WeakMap" => return Some(ESVersion::ES6),
                            "WeakSet" => return Some(ESVersion::ES6),
                            "Proxy" => return Some(ESVersion::ES6),
                            "Reflect" => return Some(ESVersion::ES6),

                            // ES2016
                            "Array.prototype.includes" => return Some(ESVersion::ES2016),
                            "exponentiation" => return Some(ESVersion::ES2016),

                            // ES2017
                            "async-functions" => return Some(ESVersion::ES2017),
                            "SharedArrayBuffer" => return Some(ESVersion::ES2017),
                            "Atomics" => return Some(ESVersion::ES2017),

                            // ES2018
                            "async-iteration" => return Some(ESVersion::ES2018),
                            "object-rest" => return Some(ESVersion::ES2018),
                            "object-spread" => return Some(ESVersion::ES2018),
                            "Promise.prototype.finally" => return Some(ESVersion::ES2018),

                            // ES2019
                            "Array.prototype.flat" => return Some(ESVersion::ES2019),
                            "Array.prototype.flatMap" => return Some(ESVersion::ES2019),
                            "Object.fromEntries" => return Some(ESVersion::ES2019),
                            "optional-catch-binding" => return Some(ESVersion::ES2019),

                            // ES2020
                            "BigInt" => return Some(ESVersion::ES2020),
                            "globalThis" => return Some(ESVersion::ES2020),
                            "Promise.allSettled" => return Some(ESVersion::ES2020),
                            "optional-chaining" => return Some(ESVersion::ES2020),
                            "coalesce-expression" => return Some(ESVersion::ES2020),
                            "dynamic-import" => return Some(ESVersion::ES2020),

                            // ES2021
                            "logical-assignment-operators" => return Some(ESVersion::ES2021),
                            "Promise.any" => return Some(ESVersion::ES2021),
                            "String.prototype.replaceAll" => return Some(ESVersion::ES2021),
                            "WeakRef" => return Some(ESVersion::ES2021),
                            "FinalizationRegistry" => return Some(ESVersion::ES2021),

                            // ES2022
                            "class-fields-private" => return Some(ESVersion::ES2022),
                            "class-methods-private" => return Some(ESVersion::ES2022),
                            "class-static-block" => return Some(ESVersion::ES2022),
                            "error-cause" => return Some(ESVersion::ES2022),
                            "Object.hasOwn" => return Some(ESVersion::ES2022),
                            "Array.prototype.at" => return Some(ESVersion::ES2022),

                            // ES2023
                            "change-array-by-copy" => return Some(ESVersion::ES2023),
                            "Array.prototype.findLast" => return Some(ESVersion::ES2023),
                            "Symbol.dispose" => return Some(ESVersion::ES2023),
                            "explicit-resource-management" => return Some(ESVersion::ES2023),

                            _ => {}
                        }
                    }
                }
            }
        }
        None
    }

    fn detect_version_by_path(path: &str) -> Option<ESVersion> {
        // ES2016+
        if path.contains("es2016") || path.contains("2016") { return Some(ESVersion::ES2016); }
        if path.contains("es2017") || path.contains("2017") { return Some(ESVersion::ES2017); }
        if path.contains("es2018") || path.contains("2018") { return Some(ESVersion::ES2018); }
        if path.contains("es2019") || path.contains("2019") { return Some(ESVersion::ES2019); }
        if path.contains("es2020") || path.contains("2020") { return Some(ESVersion::ES2020); }
        if path.contains("es2021") || path.contains("2021") { return Some(ESVersion::ES2021); }
        if path.contains("es2022") || path.contains("2022") { return Some(ESVersion::ES2022); }
        if path.contains("es2023") || path.contains("2023") { return Some(ESVersion::ES2023); }

        // ES2016+ feature paths
        if path.contains("exponentiation") { return Some(ESVersion::ES2016); }
        if path.contains("async-await") || path.contains("asyncFunction") { return Some(ESVersion::ES2017); }
        if path.contains("padStart") || path.contains("padEnd") { return Some(ESVersion::ES2017); }
        if path.contains("Object.values") || path.contains("Object.entries") { return Some(ESVersion::ES2017); }
        if path.contains("Object.getOwnPropertyDescriptors") { return Some(ESVersion::ES2017); }
        if path.contains("async-iteration") { return Some(ESVersion::ES2018); }
        if path.contains("Promise.finally") || path.contains("Promise.prototype.finally") { return Some(ESVersion::ES2018); }
        if path.contains("rest-spread") || path.contains("object-rest") || path.contains("object-spread") { return Some(ESVersion::ES2018); }
        if path.contains("flatMap") || path.contains("Array.prototype.flat") { return Some(ESVersion::ES2019); }
        if path.contains("Object.fromEntries") { return Some(ESVersion::ES2019); }
        if path.contains("trimStart") || path.contains("trimEnd") { return Some(ESVersion::ES2019); }
        if path.contains("BigInt") { return Some(ESVersion::ES2020); }
        if path.contains("globalThis") { return Some(ESVersion::ES2020); }
        if path.contains("Promise.allSettled") { return Some(ESVersion::ES2020); }
        if path.contains("optional-chaining") || path.contains("coalesce-expression") { return Some(ESVersion::ES2020); }
        if path.contains("import-dynamic") || path.contains("dynamic-import") { return Some(ESVersion::ES2020); }
        if path.contains("logical-assignment") { return Some(ESVersion::ES2021); }
        if path.contains("Promise.any") { return Some(ESVersion::ES2021); }
        if path.contains("WeakRef") || path.contains("FinalizationRegistry") { return Some(ESVersion::ES2021); }
        if path.contains("class-fields") || path.contains("private-methods") { return Some(ESVersion::ES2022); }
        if path.contains("static-block") { return Some(ESVersion::ES2022); }
        if path.contains("error-cause") { return Some(ESVersion::ES2022); }
        if path.contains("Array.prototype.at") || path.contains("String.prototype.at") { return Some(ESVersion::ES2022); }
        if path.contains("toReversed") || path.contains("toSorted") || path.contains("toSpliced") { return Some(ESVersion::ES2023); }
        if path.contains("findLast") { return Some(ESVersion::ES2023); }

        // ES6 features
        if path.contains("arrow-function") { return Some(ESVersion::ES6); }
        if path.contains("/let/") || path.contains("/const/") { return Some(ESVersion::ES6); }
        if path.contains("destructuring") { return Some(ESVersion::ES6); }
        if path.contains("generator") || path.contains("yield") { return Some(ESVersion::ES6); }
        if path.contains("spread-") || path.contains("-spread") { return Some(ESVersion::ES6); }
        if path.contains("default-parameters") || path.contains("rest-parameters") { return Some(ESVersion::ES6); }
        if path.contains("template") { return Some(ESVersion::ES6); }
        if path.contains("/class/") || path.contains("class-") { return Some(ESVersion::ES6); }
        if path.contains("/Map/") || path.contains("/Set/") || path.contains("/WeakMap/") || path.contains("/WeakSet/") { return Some(ESVersion::ES6); }
        if path.contains("/Symbol/") || path.contains("Symbol.") { return Some(ESVersion::ES6); }
        if path.contains("/Proxy/") { return Some(ESVersion::ES6); }
        if path.contains("/Reflect/") { return Some(ESVersion::ES6); }
        if path.contains("/Promise/") { return Some(ESVersion::ES6); }
        if path.contains("for-of") { return Some(ESVersion::ES6); }
        if path.contains("/module/") || path.contains("/import/") { return Some(ESVersion::ES6); }

        // ES5 features
        if path.contains("strict-mode") { return Some(ESVersion::ES5); }
        if path.contains("/JSON/") { return Some(ESVersion::ES5); }
        if path.contains("Object.create") { return Some(ESVersion::ES5); }
        if path.contains("Object.defineProperty") || path.contains("Object.defineProperties") { return Some(ESVersion::ES5); }
        if path.contains("Object.getPrototypeOf") { return Some(ESVersion::ES5); }
        if path.contains("Object.keys") { return Some(ESVersion::ES5); }
        if path.contains("Object.getOwnPropertyNames") { return Some(ESVersion::ES5); }
        if path.contains("Object.getOwnPropertyDescriptor") { return Some(ESVersion::ES5); }
        if path.contains("Object.preventExtensions") || path.contains("Object.seal") || path.contains("Object.freeze") { return Some(ESVersion::ES5); }
        if path.contains("Array.isArray") { return Some(ESVersion::ES5); }
        if path.contains("Array.prototype.every") || path.contains("Array.prototype.some") { return Some(ESVersion::ES5); }
        if path.contains("Array.prototype.filter") || path.contains("Array.prototype.forEach") { return Some(ESVersion::ES5); }
        if path.contains("Array.prototype.indexOf") || path.contains("Array.prototype.lastIndexOf") { return Some(ESVersion::ES5); }
        if path.contains("Array.prototype.map") || path.contains("Array.prototype.reduce") { return Some(ESVersion::ES5); }
        if path.contains("Function.prototype.bind") { return Some(ESVersion::ES5); }
        if path.contains("String.prototype.trim") { return Some(ESVersion::ES5); }
        if path.contains("Date.now") || path.contains("Date.toISOString") { return Some(ESVersion::ES5); }

        // ES3 features
        if path.contains("/S15.10/") || path.contains("/RegExp/") { return Some(ESVersion::ES3); }
        if path.contains("/try/") || path.contains("/catch/") || path.contains("/finally/") { return Some(ESVersion::ES3); }
        if path.contains("do-while") { return Some(ESVersion::ES3); }

        // Sputnik tests - ES1/ES3 era
        if path.contains("/S11.") || path.contains("/S12.") || path.contains("/S13.") ||
           path.contains("/S14.") || path.contains("/S15.") {
            if path.contains("/S15.10/") {
                return Some(ESVersion::ES3); // RegExp is ES3
            }
            return Some(ESVersion::ES1);
        }

        None
    }

    pub fn parse(code: &String, path: &str) -> (bool, bool, String, ESVersion) {
        let mut no_strict = false;
        let mut negative = false;
        let mut negative_type = String::from("");

        let start = code.find("/*---");
        if let Some(start) = start {
            let end = code.find("---*/");
            if let Some(end) = end {
                let config = &code[start + 5..end];
                let docs = YamlLoader::load_from_str(config);
                if let Ok(docs) = docs {

                    if let Yaml::Array(arr) = &docs[0]["flags"] {
                        for flag in arr.iter() {
                            if let Some(str) = flag.as_str() {
                                match str {
                                    "noStrict" => {
                                        no_strict = true;
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Yaml::BadValue = docs[0]["negative"] {
                    } else {
                        negative = true;
                        let negative_type_value = docs[0]["negative"]["type"].as_str();
                        if let Some(negative_type_item) = negative_type_value {
                            negative_type = String::from(negative_type_item);
                        }
                    }
                }
            }
        }

        // 检测版本 - 优先元数据，然后路径
        let version = Test262File::detect_version_by_meta(code)
            .or_else(|| Test262File::detect_version_by_path(path))
            .unwrap_or(ESVersion::Unknown);

        return (no_strict, negative, negative_type, version);
    }
}

fn load_harness(path: &str) -> String {
    let mut file = File::open(format!("test262/{}", path)).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    return code;
}

fn make_dir(dir: &String) -> PathBuf {
    Path::new(
        &env::current_dir().unwrap()
    ).join(dir)
}

fn run_single_test(file: &Test262File, preload_code: &str) -> bool {
    let result = panic::catch_unwind(|| {
        let mut jsi = JSI::new();
        jsi.set_strict(!file.no_strict);
        jsi.run(format!("{}\n{}", preload_code, file.code))
    });

    if result.is_err() {
        return false;
    }

    if let Ok(inner_result) = result {
        match inner_result {
            Err(jsi_error) => {
                if file.negative {
                    let error = jsi_error.error_type.to_string();
                    if file.negative_type.len() > 0 && error != file.negative_type {
                        false
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            Ok(_) => !file.negative
        }
    } else {
        false
    }
}

fn collect_all_files(dir: &str, ignore_list: &Vec<PathBuf>) -> Vec<Test262File> {
    let mut results = Vec::new();
    let dir_path = make_dir(&String::from(dir));
    let paths = fs::read_dir(&dir_path).unwrap();

    for entry in paths.filter_map(|e| e.ok()) {
        let path = entry.path();
        if ignore_list.contains(&path) {
            continue;
        }
        let md = fs::metadata(&path).unwrap();
        if md.is_dir() {
            results.extend(collect_all_files(path.to_str().unwrap(), ignore_list));
        } else {
            let name = path.file_name().unwrap().to_str().unwrap();
            if name.ends_with(".js") {
                results.push(Test262File::new(
                    String::from(name),
                    String::from(path.to_str().unwrap()),
                ));
            }
        }
    }
    results
}

#[derive(Debug, Serialize)]
struct SummaryResult {
    version: &'static str,
    total: usize,
    passed: usize,
    pass_rate: String,
}

#[test]
fn test_test262_by_version() {
    thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            test_test262_by_version_inner();
        })
        .unwrap()
        .join()
        .unwrap();
}

fn test_test262_by_version_inner() {
    println!("=== Test262 by ECMAScript Version ===\n");

    let preload_list = vec![
        load_harness("harness/assert.js"),
        load_harness("harness/sta.js"),
        load_harness("harness/compareArray.js"),
    ];
    let preload = preload_list.join("\n");

    let ignore_list: Vec<PathBuf> = vec![
        make_dir(&String::from("test262/test/annexB")),
        make_dir(&String::from("test262/test/intl402")),
    ];

    println!("Collecting test files...");
    let files = collect_all_files("test262/test", &ignore_list);
    println!("Total files collected: {}\n", files.len());

    // 按版本分组
    let mut version_map: HashMap<ESVersion, VersionResult> = HashMap::new();
    for file in &files {
        version_map.entry(file.version)
            .or_insert_with(|| VersionResult::new(file.version));
    }

    // 按版本顺序
    let version_order = vec![
        ESVersion::ES1,
        ESVersion::ES3,
        ESVersion::ES5,
        ESVersion::ES6,
        ESVersion::ES2016,
        ESVersion::ES2017,
        ESVersion::ES2018,
        ESVersion::ES2019,
        ESVersion::ES2020,
        ESVersion::ES2021,
        ESVersion::ES2022,
        ESVersion::ES2023,
        ESVersion::Unknown,
    ];

    // 运行测试并收集结果
    let mut summary = Vec::new();
    let mut total_cases = 0;
    let mut total_passed = 0;

    for version in version_order {
        let result = version_map.get_mut(&version);
        if let Some(vr) = result {
            println!("Running {} tests... ({} files)", version.to_str(), files.iter().filter(|f| f.version == version).count());

            for file in files.iter().filter(|f| f.version == version) {
                let passed = run_single_test(file, &preload);
                vr.add_result(file.path.clone(), passed);
            }

            let pass_rate = if vr.cases > 0 {
                format!("{:.2}%", (vr.passed as f64 / vr.cases as f64) * 100.0)
            } else {
                "N/A".to_string()
            };

            println!("{}: {} / {} ({})\n", version.to_str(), vr.passed, vr.cases, pass_rate);

            summary.push(SummaryResult {
                version: version.to_str(),
                total: vr.cases,
                passed: vr.passed,
                pass_rate,
            });

            total_cases += vr.cases;
            total_passed += vr.passed;
        }
    }

    // 输出 ES1 失败的测试
    if let Some(es1_result) = version_map.get(&ESVersion::ES1) {
        println!("{}", "=".repeat(60));
        println!("ES1 FAILED TESTS ({}):", es1_result.files.iter().filter(|(_, p)| !p).count());
        println!("{}", "=".repeat(60));
        for (i, (path, passed)) in es1_result.files.iter().filter(|(_, p)| !p).take(100).enumerate() {
            println!("{:3}. {}", i + 1, path);
        }
        println!();
    }

    // 输出总结
    println!("{}", "=".repeat(60));
    println!("SUMMARY");
    println!("{}", "=".repeat(60));
    println!("{:<10} {:>10} {:>10} {:>12}", "Version", "Total", "Passed", "Pass Rate");
    println!("{}", "-".repeat(60));

    for s in &summary {
        println!("{:<10} {:>10} {:>10} {:>12}", s.version, s.total, s.passed, s.pass_rate);
    }

    println!("{}", "-".repeat(60));
    let total_rate = if total_cases > 0 {
        format!("{:.2}%", (total_passed as f64 / total_cases as f64) * 100.0)
    } else {
        "N/A".to_string()
    };
    println!("{:<10} {:>10} {:>10} {:>12}", "TOTAL", total_cases, total_passed, total_rate);

    // 保存结果
    let serialized = serde_json::to_string_pretty(&summary).unwrap();
    let mut file = File::create("./tests/test262_by_version_result.json").unwrap();
    file.write_all(serialized.as_bytes()).unwrap();
    println!("\nResults saved to ./tests/test262_by_version_result.json");
}
