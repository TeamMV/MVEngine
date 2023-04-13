pub mod assets;

#[cfg(test)]
mod tests {
    use include_dir::{Dir, include_dir};
    use crate::assets::*;

    #[test]
    fn it_works() {
        static DIR: Dir = include_dir!("assets/");
        let manager = AssetManager::manual(DIR.clone());

    }
}
