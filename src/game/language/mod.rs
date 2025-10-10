use hashbrown::HashMap;

pub struct Language {
    lookup: HashMap<String, String>,
}

impl Language {
    pub fn parse(inp: &str) -> Self {
        let lines = inp.lines();
        let mut map = HashMap::new();

        for line in lines {
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.to_string(), value.to_string());
            }
        }
        Language { lookup: map }
    }

    pub fn lookup(&self, key: &str) -> Option<&String> {
        self.lookup.get(key)
    }

    pub fn maybe_lookup(&self, key: &str) -> String {
        self.lookup(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn has(&self, key: &str) -> bool {
        self.lookup.contains_key(key)
    }
}
