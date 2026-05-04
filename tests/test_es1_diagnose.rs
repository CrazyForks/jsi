use std::{
    path::{Path, PathBuf},
    env, fs::{self, File},
    io::{Write, Read},
    panic, thread,
};
use jsi::JSI;
use yaml_rust::{YamlLoader, Yaml};

#[derive(Debug)]
struct TestCase {
    path: String,
    code: String,
    no_strict: bool,
    negative: bool,
    negative_type: String,
}

fn parse_test_file(path: &str) -> TestCase {
    let mut file = File::open(path).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();

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
                            if str == "noStrict" {
                                no_strict = true;
                            }
                        }
                    }
                }
                if let Yaml::BadValue = docs[0]["negative"] {
                } else {
                    negative = true;
                    if let Some(t) = docs[0]["negative"]["type"].as_str() {
                        negative_type = String::from(t);
                    }
                }
            }
        }
    }

    TestCase {
        path: String::from(path),
        code,
        no_strict,
        negative,
        negative_type,
    }
}

fn is_es1_test(path: &str) -> bool {
    if path.contains("/S15.10/") || path.contains("/RegExp/") {
        return false; // ES3
    }
    if path.contains("/S11.") || path.contains("/S12.") || path.contains("/S13.") ||
       path.contains("/S14.") || (path.contains("/S15.") && !path.contains("/S15.10")) {
        return true;
    }
    false
}

fn collect_es1_files(dir: &str, ignore_list: &Vec<PathBuf>, results: &mut Vec<TestCase>) {
    let dir_path = Path::new(&env::current_dir().unwrap()).join(dir);
    let paths = fs::read_dir(&dir_path).unwrap();

    for entry in paths.filter_map(|e| e.ok()) {
        let path = entry.path();
        if ignore_list.contains(&path) {
            continue;
        }
        let md = fs::metadata(&path).unwrap();
        if md.is_dir() {
            collect_es1_files(path.to_str().unwrap(), ignore_list, results);
        } else {
            let name = path.file_name().unwrap().to_str().unwrap();
            if name.ends_with(".js") && is_es1_test(path.to_str().unwrap()) {
                results.push(parse_test_file(path.to_str().unwrap()));
            }
        }
    }
}

fn load_harness(path: &str) -> String {
    let mut file = File::open(format!("test262/{}", path)).unwrap();
    let mut code = String::new();
    file.read_to_string(&mut code).unwrap();
    return code;
}

fn run_test(test: &TestCase, preload: &str) -> (bool, Option<String>) {
    let result = panic::catch_unwind(|| {
        let mut jsi = JSI::new();
        jsi.set_strict(!test.no_strict);
        jsi.run(format!("{}\n{}", preload, test.code))
    });

    if result.is_err() {
        return (false, Some("Panic".to_string()));
    }

    match result.unwrap() {
        Err(jsi_error) => {
            if test.negative {
                let error = jsi_error.error_type.to_string();
                if test.negative_type.len() > 0 && error != test.negative_type {
                    (false, Some(format!("Wrong error: expected {}, got {}", test.negative_type, error)))
                } else {
                    (true, None)
                }
            } else {
                (false, Some(format!("Error: {}", jsi_error.error_type.to_string())))
            }
        }
        Ok(_) => {
            if test.negative {
                (false, Some("Expected error but none occurred".to_string()))
            } else {
                (true, None)
            }
        }
    }
}

#[test]
fn test_es1_diagnose() {
    thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            diagnose_es1();
        })
        .unwrap()
        .join()
        .unwrap();
}

fn diagnose_es1() {
    println!("=== ES1 Test Diagnosis ===\n");

    let preload_list = vec![
        load_harness("harness/assert.js"),
        load_harness("harness/sta.js"),
    ];
    let preload = preload_list.join("\n");

    let ignore_list: Vec<PathBuf> = vec![
        Path::new(&env::current_dir().unwrap()).join("test262/test/annexB"),
        Path::new(&env::current_dir().unwrap()).join("test262/test/intl402"),
    ];

    println!("Collecting ES1 test files...");
    let mut tests = Vec::new();
    collect_es1_files("test262/test", &ignore_list, &mut tests);
    println!("Total ES1 tests: {}\n", tests.len());

    // 按目录分组
    let mut by_category: std::collections::HashMap<String, Vec<(TestCase, Option<String>)>> = std::collections::HashMap::new();

    let mut passed = 0;
    let mut failed = 0;

    for test in tests {
        let (ok, error) = run_test(&test, &preload);
        if ok {
            passed += 1;
        } else {
            failed += 1;
        }

        // 提取分类目录
        let parts: Vec<&str> = test.path.split('/').collect();
        let category = if parts.len() >= 3 {
            format!("{}/{}", parts[parts.len() - 3], parts[parts.len() - 2])
        } else {
            "unknown".to_string()
        };

        by_category.entry(category)
            .or_insert_with(Vec::new)
            .push((test, error));
    }

    println!("Total: {} Passed: {} Failed: {}\n", passed + failed, passed, failed);

    // 按失败率排序的分类输出
    let mut categories: Vec<_> = by_category.into_iter().collect();
    categories.sort_by(|a, b| {
        let a_fail = a.1.iter().filter(|(_, e)| e.is_some()).count();
        let b_fail = b.1.iter().filter(|(_, e)| e.is_some()).count();
        b_fail.cmp(&a_fail)
    });

    let mut report = Vec::new();

    println!("{}", "=".repeat(80));
    println!("FAILURE ANALYSIS BY CATEGORY");
    println!("{}", "=".repeat(80));

    for (cat, tests) in &categories {
        let total = tests.len();
        let cat_failed = tests.iter().filter(|(_, e)| e.is_some()).count();
        if cat_failed > 0 {
            let rate = (cat_failed as f64 / total as f64) * 100.0;
            println!("{:<30} total: {:4} failed: {:4} ({:5.1}%)", cat, total, cat_failed, rate);

            // 收集这个分类的前10个失败案例
            let mut failures = Vec::new();
            for (test, error) in tests {
                if let Some(err) = error {
                    failures.push((&test.path, err));
                }
            }
            failures.truncate(5);
            for (path, err) in failures {
                let short_path: Vec<&str> = path.split('/').rev().take(2).collect();
                let short_path = short_path.into_iter().rev().collect::<Vec<_>>().join("/");
                println!("  - {}: {}", short_path, err);
                report.push((cat.clone(), short_path, err.clone()));
            }
            println!();
        }
    }

    // 保存详细报告
    let json_report = serde_json::to_string_pretty(&report).unwrap();
    let mut file = File::create("./tests/es1_failure_report.json").unwrap();
    file.write_all(json_report.as_bytes()).unwrap();
    println!("\nDetailed report saved to ./tests/es1_failure_report.json");
}
