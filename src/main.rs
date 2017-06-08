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
                if let Some(filename) = path.file_name().and_then(|name| {
                    name.to_str()
                }) {
                    if filename == "documentation.json" {
                        cb(&path);
                    }
                }
            }
        }
    }
}

fn print_value(value: &Value) {
    let mut out = term::stdout().unwrap();

    out.fg(term::color::BRIGHT_BLUE).unwrap();
    write!(out, "{}", value["name"].as_str().unwrap()).unwrap();
    write!(out, " : ").unwrap();
    out.fg(term::color::GREEN).unwrap();
    write!(out, "{}", value["type"].as_str().unwrap()).unwrap();
    write!(out, "\n").unwrap();
    out.fg(term::color::CYAN).unwrap();
    write!(out, "    {}", value["comment"].as_str().unwrap()).unwrap();

    out.reset().unwrap();
}

fn read_documentation(path: &Path) -> Result<Value, Box<Error>> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let v: Value = serde_json::from_str(&data)?;
    Ok(v)
}

fn find_needle_in_haystack<'a>(search: &str,
                               haystack: &'a Value)
                               -> Vec<&'a Value> {
    use serde_json::Value::*;
    match *haystack {
        Object(ref hay_map) => {
            if hay_map.get("name").cloned() ==
               Some(String(search.to_string())) {
                vec![haystack]
            } else {
                hay_map
                    .iter()
                       .flat_map(|(_, value)| {
                           match *value {
                               Array(ref json_array) => {
                                   json_array
                                       .iter()
                                       .flat_map(|ja| {
                                           find_needle_in_haystack(search, ja)
                                       })
                                       .collect::<Vec<_>>()
                               }
                               _ => vec![],
                           }  
                       })
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

fn search_for(search: &str) -> Result<(), Box<Error>> {
    println!("Searching for: {}", search);
    println!("");

    let cwd = env::current_dir()?;
    run_on_all_the_documentation(&cwd,
                                 &|doc_path| if let Ok(docs) =
        read_documentation(doc_path) {
                                     let results =
                                         find_needle_in_haystack(search, &docs);
                                     for result in results {
                                         print_value(result);
                                     }
                                 });
    // let path = "C:/Users/micro/Code/play/games/OneRoom/elm-stuff/packages/evancz/elm-graphics/1.0.1/documentation.json";
    // let graphics = read_documentation(Path::new(path))?;
    Ok(())
}

fn main() {
    if let Some(search) = env::args().nth(1) {
        let _ = search_for(&search);
    } else {
        println!("Please provide an argument to search docs for");
    }
}
