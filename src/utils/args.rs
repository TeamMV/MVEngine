use crate::utils::Expect2;
use hashbrown::HashMap;
use std::str::FromStr;

pub struct ParsedArgs {
    args: HashMap<String, String>,
}

impl ParsedArgs {
    pub fn parse(mut arg_iter: impl Iterator<Item = String>) -> Self {
        let mut arg = arg_iter.next();
        let mut map = HashMap::new();
        let mut last_key = None;
        while let Some(string) = &arg {
            if last_key.is_none() {
                last_key = Some(string.clone());
            } else {
                map.insert(last_key.unwrap(), string.clone());
                last_key = None;
            }
            arg = arg_iter.next();
        }
        Self { args: map }
    }

    pub fn get(&self, key: &str) -> &str {
        self.args
            .get(key)
            .expect(&format!("Argument '{key}' does not exist!"))
    }

    pub fn try_get(&self, key: &str) -> Option<&str> {
        self.args.get(key).map(|s| s.as_str())
    }

    pub fn get_as<T: FromStr>(&self, key: &str) -> T {
        let s = self
            .args
            .get(key)
            .expect(&format!("Argument '{key}' does not exist!"));
        T::from_str(s).expect2("The argument '{key}' couldnt be parsed into T!")
    }

    pub fn try_get_as<T: FromStr>(&self, key: &str) -> Option<T> {
        if let Some(v) = self.args.get(key) {
            return T::from_str(v).ok();
        }
        None
    }
}
