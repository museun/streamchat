use std::collections::HashMap;

pub struct Args(pub HashMap<String, String>);
impl Args {
    pub fn parse(args: &[String]) -> Option<Self> {
        let mut map = HashMap::new();
        for chunk in args.chunks(2) {
            if chunk.len() == 2 {
                map.insert(chunk[0].clone(), chunk[1].clone());
            } else {
                return None;
            }
        }
        Some(Args(map))
    }

    pub fn get(&self, k: &str, def: &'static str) -> String {
        self.0.get(k).cloned().unwrap_or_else(|| def.to_owned())
    }

    pub fn get_as<T>(&self, k: &str, def: T, f: fn(&String) -> Option<T>) -> T {
        self.0.get(k).and_then(|s| f(s)).unwrap_or_else(|| def)
    }
}
