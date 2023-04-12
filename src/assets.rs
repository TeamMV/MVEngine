use std::collections::HashMap;
use include_dir::*;

pub struct AssetManager {
    textures: HashMap<String, TextureRegion>,
    sounds: HashMap<String, Sound>,
}

impl AssetManager {
    pub fn new(dir: Dir) -> Self {
        let config = dir.get_file("assets.dat").expect("Asset data file not found. Compile an assets folder!").clone();
        let mut file_map: HashMap<String, File> = HashMap::new();
        Self::files_deep(dir).into_iter().map(|file| (file.path().to_string(), file)).for_each(|(pair, file)| {
            file_map.insert(pair, file);
        });
        //parse config, map files to assets
        drop(file_map);
        todo!()
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

pub struct Texture {

}

pub struct TextureRegion {
    texture: Texture,
    x: u32,
    y: u32,
    width: u32,
    height: u32
}

pub struct Sound {

}