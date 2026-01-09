use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, PartialEq, Eq)]
pub enum FileType {
    Stl,
    ZippedStl,
    Obj,
    ZippedObj,
    Gcode,
    ZippedGcode,
    Step,
    ZippedStep,
    Threemf,
    Unknown
}


#[derive(Serialize, Clone)]
pub struct Blob {
    pub id: i64,
    pub sha256: String,
    pub filetype: String,
    pub size: i64,
    pub added: String,
    pub disk_path: Option<String>
}

impl Blob {
    pub fn to_file_type(&self) -> FileType {
        FileType::from_extension(&self.filetype)
    }
}

impl FileType {
    pub fn from_pathbuf(path: &PathBuf) -> FileType {
        match path.extension() {
            Some(ext) => FileType::from_extension(&ext.to_string_lossy()),
            None => FileType::Unknown
        }
    }

    pub fn from_extension(extension: &str) -> FileType {
        match extension.to_lowercase().as_str() {
            f if f.ends_with("stl") => FileType::Stl,
            f if f.ends_with("stl.zip") => FileType::ZippedStl,
            f if f.ends_with("obj") => FileType::Obj,
            f if f.ends_with("obj.zip") => FileType::ZippedObj,
            f if f.ends_with("gcode") => FileType::Gcode,
            f if f.ends_with("gcode.zip") => FileType::ZippedGcode,
            f if f.ends_with("step") => FileType::Step,
            f if f.ends_with("stp") => FileType::Step,
            f if f.ends_with("step.zip") => FileType::ZippedStep,
            f if f.ends_with("stp.zip") => FileType::ZippedStep,
            f if f.ends_with("3mf") => FileType::Threemf,
            _ => FileType::Unknown
        }
    }

    pub fn to_extension(&self) -> String {
        match self {
            FileType::Stl => "stl",
            FileType::ZippedStl => "stl.zip",
            FileType::Obj => "obj",
            FileType::ZippedObj => "obj.zip",
            FileType::Gcode => "gcode",
            FileType::ZippedGcode => "gcode.zip",
            FileType::Step => "step",
            FileType::ZippedStep => "step.zip",
            FileType::Threemf => "3mf",
            FileType::Unknown => panic!("Cannot convert Unknown FileType to extension"),
        }.to_string()
    }

    pub fn is_zipped(&self) -> bool {
        match self {
            FileType::ZippedStl => true,
            FileType::ZippedObj => true,
            FileType::ZippedGcode => true,
            FileType::ZippedStep => true,
            _ => false
        }
    }

    pub fn is_stl(&self) -> bool {
        match self {
            FileType::Stl => true,
            FileType::ZippedStl => true,
            _ => false
        }
    }

    pub fn is_obj(&self) -> bool {
        match self {
            FileType::Obj => true,
            FileType::ZippedObj => true,
            _ => false
        }
    }

    pub fn is_3mf(&self) -> bool {
        match self {
            FileType::Threemf => true,
            _ => false
        }
    }

    pub fn is_step(&self) -> bool {
        match self {
            FileType::Step => true,
            FileType::ZippedStep => true,
            _ => false
        }
    }

    pub fn is_gcode(&self) -> bool {
        match self {
            FileType::Gcode => true,
            FileType::ZippedGcode => true,
            _ => false
        }
    }

    pub fn is_unsupported(&self) -> bool {
        match self {
            FileType::Unknown => true,
            _ => false
        }
    }

    pub fn is_zippable(&self) -> bool {
        match self {
            FileType::Stl => true,
            FileType::Obj => true,
            FileType::Step => true,
            FileType::Gcode => true,
            _ => false
        }
    }

    pub fn to_zip(&self) -> FileType {
        match self {
            FileType::Stl => FileType::ZippedStl,
            FileType::Obj => FileType::ZippedObj,
            FileType::Step => FileType::ZippedStep,
            FileType::Gcode => FileType::ZippedGcode,
            _ => self.clone()
        }
    }

    pub fn from_zip(&self) -> FileType {
        match self {
            FileType::ZippedStl => FileType::Stl,
            FileType::ZippedObj => FileType::Obj,
            FileType::ZippedStep => FileType::Step,
            FileType::ZippedGcode => FileType::Gcode,
            _ => self.clone()
        }
    }

    pub fn is_supported_by_thumbnail_gen(&self) -> bool {
        match self {
            FileType::Stl => true,
            FileType::ZippedStl => true,
            FileType::Obj => true,
            FileType::ZippedObj => true,
            FileType::Gcode => true,
            FileType::ZippedGcode => true,
            FileType::Step => true,
            FileType::ZippedStep => true,
            FileType::Threemf => true,
            _ => false
        }
    }

    pub fn is_importable(&self) -> bool {
        match self {
            FileType::Stl => true,
            FileType::Obj => true,
            FileType::Gcode => true,
            FileType::Step => true,
            FileType::Threemf => true,
            _ => false
        }
    }
}

impl<'de> Deserialize<'de> for FileType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(FileType::from_extension(s))
    }
}