use std::collections::HashMap;
use include_dir::*;

pub struct AssetManager {

}

impl AssetManager {
    pub fn automatic(dir: Dir) -> AutomaticAssetManager {
        let config = dir.get_file("assets.dat").expect("Automatic asset manager requires assets.dat file!").clone();
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn new(dir: Dir) -> SemiAutomaticAssetManager {
        let config = dir.get_file("assets.dat").map(|f| f.clone());
        let mut file_map = Self::map(dir);
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn manual(dir: Dir) -> ManualAssetManager {
        let mut file_map = Self::map(dir);
        drop(file_map);
        todo!()
    }

    fn map(dir: Dir) -> HashMap<String, File> {
        let mut file_map: HashMap<String, File> = HashMap::new();
        Self::files_deep(dir).into_iter().map(|file| {
            (file.path().to_path_buf().into_os_string().into_string().unwrap(), file)
        }).for_each(|(pair, file)| {
            file_map.insert(pair, file);
        });
        file_map
    }

    fn files_deep(dir: Dir) -> Vec<File> {
        let mut files_deep = dir
            .dirs()
            .map(ToOwned::to_owned)
            .flat_map(Self::files_deep)
            .collect::<Vec<_>>();
        files_deep
            .append(dir
                .files()
                .into_iter()
                .map(File::to_owned)
                .collect::<Vec<_>>().as_mut());
        files_deep
    }
}

pub struct AutomaticAssetManager {
    manager: AssetManager
}

impl AutomaticAssetManager {

}

pub struct SemiAutomaticAssetManager {
    manager: AssetManager
}

impl SemiAutomaticAssetManager {

}

pub struct ManualAssetManager {
    manager: AssetManager
}

impl ManualAssetManager {

}