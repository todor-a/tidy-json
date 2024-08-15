use rand::Rng;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let test_files_dir = Path::new("test_files");
    fs::create_dir_all(test_files_dir)?;

    let num_files = 5; // You can change this to generate more or fewer files

    for i in 0..num_files {
        let file_name = format!("test_file_{}.json", i + 1);
        let file_path = test_files_dir.join(file_name);
        let json_data = generate_random_json();

        let mut file = File::create(file_path)?;
        let formatted_json = serde_json::to_string_pretty(&json_data)?;
        file.write_all(formatted_json.as_bytes())?;
    }

    println!(
        "Generated {} test files in the 'test_files' directory",
        num_files
    );
    Ok(())
}

fn generate_random_json() -> Value {
    let mut rng = rand::thread_rng();

    // Create a BTreeMap (sorted) with our data
    let mut sorted_map: BTreeMap<String, Value> = BTreeMap::new();
    sorted_map.insert("id".to_string(), json!(rng.gen::<u32>()));
    sorted_map.insert(
        "name".to_string(),
        json!(generate_random_string(rng.gen_range(5..15))),
    );
    sorted_map.insert("age".to_string(), json!(rng.gen_range(18..80)));
    sorted_map.insert("is_active".to_string(), json!(rng.gen_bool(0.5)));
    sorted_map.insert(
        "balance".to_string(),
        json!((rng.gen::<f64>() * 10000.0).round() / 100.0),
    );
    sorted_map.insert(
        "tags".to_string(),
        json!(generate_random_array(rng.gen_range(1..5))),
    );
    sorted_map.insert(
        "friends".to_string(),
        json!(generate_random_array(rng.gen_range(1..3))),
    );

    let mut address_map: BTreeMap<String, Value> = BTreeMap::new();
    address_map.insert(
        "street".to_string(),
        json!(generate_random_string(rng.gen_range(10..20))),
    );
    address_map.insert(
        "city".to_string(),
        json!(generate_random_string(rng.gen_range(6..12))),
    );
    address_map.insert(
        "country".to_string(),
        json!(generate_random_string(rng.gen_range(4..8))),
    );

    sorted_map.insert(
        "address".to_string(),
        Value::Object(Map::from_iter(
            address_map.into_iter().collect::<HashMap<_, _>>(),
        )),
    );

    // Convert the BTreeMap to a HashMap to randomize the order
    let unsorted_map: HashMap<_, _> = sorted_map.into_iter().collect();

    // Convert the HashMap back to a JSON Value
    Value::Object(Map::from_iter(unsorted_map))
}

fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn generate_random_array(length: usize) -> Vec<String> {
    (0..length)
        .map(|_| generate_random_string(rand::thread_rng().gen_range(3..8)))
        .collect()
}
