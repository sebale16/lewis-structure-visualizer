use serde::de::Error;

use crate::solve::Model;

struct Entry {
    name: String,
    
}

impl Model {
    // write Model to json file for Blender import
    pub fn write_to_json(&self, path: &str) -> Result<(), Box<dyn Error>> { 
        // Result type returns Ok or Err; in this case, we don't care about Ok; 
        // dyn Error captures all errors that have the Error trait but it has unknown size
        // Box gives us an owned pointer that points to the Error and has a fixed size

    }
}
