use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use bson::{Document, Bson};
use std::convert::From;


// Function to load a BSON hashmap from a file
pub fn load_bson_hashmap(filename: &str) -> io::Result<HashMap<String, Vec<f64>>> {
    let file = OpenOptions::new().read(true).write(true).create(true).open(filename)?;
    let doc = Document::from_reader(file)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut hashmap = HashMap::new();
    for (key, value) in doc {
        if let Ok(key) = bson::from_bson(Bson::String(key)) {
            if let Ok(value) = bson::from_bson(value) {
                hashmap.insert(key, value);
            }
        }
    }
    Ok(hashmap)
}

pub fn save_bson_hashmap(hashmap: &HashMap<String, Vec<f64>>, filename: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(filename)?;
    let mut doc = Document::new();
    for (key, value) in hashmap {
        doc.insert(key.clone(), bson::to_bson(value)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?);
    }
    file.write_all(&bson::to_vec(&doc)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?)?;
    Ok(())
}

// Function to load the BSON hashmap initially
pub fn load_initial_hashmap(filename: &str) -> HashMap<String, Vec<f64>> {
    match load_bson_hashmap(filename) {
        Ok(hashmap) => hashmap,
        Err(_) => HashMap::new(), // If loading fails, return an empty hashmap
    }
}