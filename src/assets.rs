use std::collections::HashMap;
use std::marker::PhantomData;
use include_dir::*;

pub trait ManagerType {}
pub trait AutomaticType : ManagerType {}
pub trait SemiAutomaticType : AutomaticType + ManualType {}
pub trait ManualType : ManagerType {}
struct Automatic;
struct SemiAutomatic;
struct Manual;
impl ManagerType for Manual {}
impl ManualType for Manual {}
impl ManagerType for SemiAutomatic {}
impl AutomaticType for SemiAutomatic {}
impl ManualType for SemiAutomatic {}
impl SemiAutomaticType for SemiAutomatic {}
impl ManagerType for Automatic {}
impl AutomaticType for Automatic {}

pub struct AssetManager<T: ManagerType> {
    phantom: PhantomData<T>,
}

impl<T> AssetManager<T> where T: ManagerType {
    pub fn automatic(dir: Dir) -> AssetManager<Automatic> {
        let config = dir.get_file("assets.dat").expect("Asset data file not found. Compile an assets folder!").clone();
        let mut file_map: HashMap<String, File> = HashMap::new();
        Self::files_deep(dir).into_iter().map(|file| (file.path().to_string(), file)).for_each(|(pair, file)| {
            file_map.insert(pair, file);
        });
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn semi_automatic(dir: Dir) -> AssetManager<SemiAutomatic> {
        let config = dir.get_file("assets.dat").expect("Asset data file not found. Compile an assets folder!").clone();
        let mut file_map: HashMap<String, File> = HashMap::new();
        Self::files_deep(dir).into_iter().map(|file| (file.path().to_string(), file)).for_each(|(pair, file)| {
            file_map.insert(pair, file);
        });
        //parse config, map files to assets
        drop(file_map);
        todo!()
    }

    pub fn manual(dir: Dir) -> AssetManager<Manual> {
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

impl<T> AssetManager<T> where T: AutomaticType {
    pub fn parse(&self) {
        todo!()
    }
}

impl<T> AssetManager<T> where T: ManualType {
    pub fn add(&mut self) {
        todo!()
    }
}