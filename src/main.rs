extern crate serde_json;
extern crate term;

use serde_json::Value;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn run_on_all_the_documentation(dir: &Path, cb: &Fn(&Path)) {
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                run_on_all_the_documentation(&path, cb);
            } else {
                if let Some(filename) = path.file_name().and_then(
                    |name| name.to_str(),
                )
                {
                    if filename == "documentation.json" {
                        cb(&path);
                    }
                }
            }
        }
    }
}

fn print_value(doc_name: &str, value: &Value) {
    let mut out = term::stdout().unwrap();

    out.fg(term::color::BRIGHT_YELLOW).unwrap();
    write!(out, "{}:", doc_name).unwrap();
    write!(out, "\n").unwrap();
    out.fg(term::color::BRIGHT_BLUE).unwrap();
    write!(out, "{}", value["name"].as_str().unwrap_or("*no name*")).unwrap();
    write!(out, " : ").unwrap();
    out.fg(term::color::GREEN).unwrap();
    write!(out, "{}", value["type"].as_str().unwrap_or("*no type*")).unwrap();
    write!(out, "\n").unwrap();
    out.fg(term::color::CYAN).unwrap();
    write!(
        out,
        "    {}",
        value["comment"].as_str().unwrap_or("*no comment*")
    ).unwrap();

    out.reset().unwrap();
}

fn read_documentation(path: &Path) -> Result<Value, Box<Error>> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let v: Value = serde_json::from_str(&data)?;
    Ok(v)
}

fn go_into_arrays<'a>(search: &str, value: &'a Value) -> Vec<&'a Value> {
    use serde_json::Value::*;
    if let Array(ref json_array) = *value {
        json_array
            .iter()
            .flat_map(|ja| find_needle_in_haystack(search, ja))
            .collect::<Vec<_>>()
    } else {
        vec![]
    }
}

fn find_needle_in_haystack<'a>(
    search: &str,
    haystack: &'a Value,
) -> Vec<&'a Value> {
    use serde_json::Value::*;
    match *haystack {
        Object(ref hay_map) => {
            if hay_map.get("name").cloned() ==
                Some(String(search.to_string()))
            {
                vec![haystack]
            } else if hay_map.get("type").cloned().and_then(|typen| {
                typen.as_str().map(|types| types.contains(search))
            }) == Some(true)
            {
                vec![haystack]
            } else {
                hay_map
                    .iter()
                    .flat_map(|(_, value)| go_into_arrays(search, value))
                    .collect::<Vec<_>>()
            }
        }
        Array(ref hay_pile) => {
            hay_pile
                .iter()
                .flat_map(|h| find_needle_in_haystack(search, h))
                .collect::<Vec<_>>()
        }
        Bool(_) | Null | Number(_) | String(_) => vec![],
    }
}

fn doc_name_from_path(path: &Path) -> Option<String> {
    let components = path.iter().rev();
    if let (Some(pkgname), Some(org)) =
        (
            components.clone().nth(2).and_then(|c| c.to_str()),
            components.clone().nth(3).and_then(|c| c.to_str()),
        )
    {
        Some(format!("{}/{}", org, pkgname))
    } else {
        None
    }
}

#[test]
fn test_doc_name_from_path() {
    assert_eq!(None, doc_name_from_path(Path::new("")));
    let example =
        Path::new("elm-stuff/packages/elm-lang/core/5.1.1/documentation.json");
    assert_eq!(
        Some(String::from("elm-lang/core")),
        doc_name_from_path(&example)
    );
}

fn find_in_documentation(search: &str, doc_path: &Path) {
    if let Ok(docs) = read_documentation(doc_path) {
        let doc_name =
            doc_name_from_path(doc_path).unwrap_or(String::from("unknown"));
        let results = find_needle_in_haystack(search, &docs);
        for result in results {
            println!();
            print_value(&doc_name, result);
        }
    }
}

fn search_for(search: &str) -> Result<(), Box<Error>> {
    println!("Searching for: {}", search);
    println!("");

    let cwd = env::current_dir()?;
    run_on_all_the_documentation(
        &cwd,
        &|doc_path| find_in_documentation(search, doc_path),
    );
    Ok(())
}

fn main() {
    if let Some(search) = env::args().nth(1) {
        let _ = search_for(&search);
    } else {
        println!("Please provide an argument to search docs for");
    }
}
