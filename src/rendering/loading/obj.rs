use regex::Regex;
use mvutils::once::Lazy;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;
use include_dir::{Dir, File};
use mvutils::{TryFromString, TryFromStringLegacy};
use crate::math::vec::Vec3;
use crate::rendering::implementation::model::material::Material;
use crate::rendering::implementation::model::model::StandaloneModel;
use crate::rendering::loading::ModelLoadingError;

pub struct OBJModelLoader<'a> {
    directory: Dir<'a>
}

impl<'a> OBJModelLoader<'a> {
    pub fn new(dir: Dir<'a>) -> Self {
        Self {
            directory: dir,
        }
    }

    fn grab_file(&self, filename: &str, file_ext: &str) -> Result<&File, ModelLoadingError> {
        let complete_name = format!("{filename}.{file_ext}");
        let path = Path::new(&complete_name);
        self.directory.get_file(path)
            .ok_or(ModelLoadingError::MissingFile(complete_name))
    }

    pub fn load_model(&self, name: &str) -> Result<StandaloneModel, ModelLoadingError> {
        let obj_file = self.grab_file(name, "obj")?;

        if let Some(contents) = obj_file.contents_utf8() {

        }
    }

    fn parse_material(&self, name: &str) -> Result<Material, ModelLoadingError> {

    }
}

enum Token {
    Command(Command),
    StrLit(String),
    FloatLit(f32),
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromString)]
enum Command {
    // ---------- .obj ----------
    #[casing(Lower)]
    O,        // o <name>
    #[casing(Lower)]
    G,        // g <group>
    #[casing(Lower)]
    V,        // v x y z
    #[casing(Lower)]
    VT,       // vt u v [w]
    #[casing(Lower)]
    VN,       // vn x y z
    #[casing(Lower)]
    F,        // f v1[/vt1/vn1] v2[/vt2/vn2] v3[/vt3/vn3]
    #[casing(Lower)]
    Usemtl,   // usemtl <name>
    #[casing(Lower)]
    Mtllib,   // mtllib <file>
    #[casing(Lower)]
    S,        // s <group smoothing>

    // ---------- .mtl ----------
    #[casing(Lower)]
    Newmtl,   // newmtl <name>
    #[casing(Lower)]
    Ka,
    #[casing(Lower)]
    Kd,
    #[casing(Lower)]
    Ks,
    #[casing(Lower)]
    Ke,
    #[casing(Lower)]
    Ns,
    #[casing(Lower)]
    Ni,
    #[casing(Lower)]
    D,
    #[casing(Lower)]
    Tr,
    #[casing(Lower)]
    Illum,

    // ---------- texture maps ----------
    // Using regex to match case-insensitive or variant spellings
    #[pattern("(?i)^map_?kd$")]
    MapKd,
    #[pattern("(?i)^map_?ka$")]
    MapKa,
    #[pattern("(?i)^map_?ks$")]
    MapKs,
    #[pattern("(?i)^(map_?bump|bump)$")]
    MapBump,
}


pub struct Tokenizer<'a> {
    source: Peekable<Chars<'a>>,
    putback: VecDeque<Token>
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            putback: VecDeque::new(),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.source.peek() {
            if c.is_whitespace() {
                self.source.next();
            } else if c == '#' {
                while let Some(ch) = self.source.next() {
                    if ch == '\n' { break; }
                }
            } else {
                break;
            }
        }
    }

    fn read_word(&mut self) -> String {
        let mut s = String::new();
        while let Some(&c) = self.source.peek() {
            if c.is_alphanumeric() || c == '.' || c == '_' {
                s.push(c);
                self.source.next();
            } else {
                break;
            }
        }
        s
    }

    fn read_number(&mut self, first: char) -> String {
        let mut s = String::new();
        s.push(first);
        while let Some(&c) = self.source.peek() {
            if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E' || c == '-' || c == '+' {
                s.push(c);
                self.source.next();
            } else {
                break;
            }
        }
        s
    }

    fn word_to_command(word: &str) -> Option<Command> {
        use Command::*;
        Some(match word {
            "o" => O, "g" => G, "v" => V, "vt" => VT, "vn" => VN, "f" => F,
            "usemtl" => Usemtl, "mtllib" => Mtllib, "s" => S,
            "newmtl" => Newmtl, "Ka" => Ka, "Kd" => Kd, "Ks" => Ks, "Ke" => Ke,
            "Ns" => Ns, "Ni" => Ni, "d" => D, "Tr" => Tr, "illum" => Illum,
            "map_Kd" => MapKd, "map_Ka" => MapKa, "map_Ks" => MapKs, "map_Bump" | "bump" => MapBump,
            _ => return None,
        })
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}