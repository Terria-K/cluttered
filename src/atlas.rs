extern crate binary_rw;
mod output;
use std::{path::{PathBuf, Path}, collections::HashMap};

use crunch::{Item, Rotation};
use image::{RgbaImage, ImageBuffer, GenericImage, GenericImageView};

use crate::error::PackerError;

use self::output::{save_output, JsonOutput, BinaryOutput, RonOutput, save_output_from, TemplateOutput};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PackerConfig {
    pub name: String,
    pub output_path: PathBuf,
    pub output_type: OutputType,
    pub folders: Vec<PathBuf>,
    pub template_path: Option<PathBuf>,
    pub options: PackerConfigOptions
}

impl PackerConfig {
    pub fn from_json(path: PathBuf) -> anyhow::Result<PackerConfig> {
        let buffer = std::fs::read(path)?;
        let packer_atlas = serde_json::from_slice::<PackerConfig>(&buffer)?;
        Ok(packer_atlas)
    }

    pub fn from_ron(path: PathBuf) -> anyhow::Result<PackerConfig> {
        let buffer = std::fs::read_to_string(path)?;
        let packer_atlas = ron::from_str::<PackerConfig>(&buffer)?;
        Ok(packer_atlas)
    }
}

#[derive(serde::Deserialize, serde::Serialize, clap::ValueEnum, Clone)]
pub enum OutputType {
    Json,
    Binary,
    Ron,
    Template
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct PackerConfigOptions {
    max_size: usize,
    show_extension: bool,
    features: Features
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct Features {
    nine_patch: bool
}

impl Default for PackerConfigOptions {
    fn default() -> Self {
        PackerConfigOptions { 
            max_size: 1024,
            show_extension: true,
            features: Features::default()
        }
    }
}

#[derive(Default, serde::Serialize)]
struct PackerAtlas {
    sheet_path: PathBuf,
    frames: HashMap<String, TextureData>
}

impl PackerAtlas {
    fn add(
        &mut self, 
        name: &str, 
        x: u32, y: u32, 
        width: u32, height: u32, 
        nine_patch: Option<Rect>
    ) {
        self.frames.insert(name.into(), TextureData {
            x, y, width, height, nine_patch
        });
    }

    fn add_sheet_path(&mut self, path: &Path) {
        self.sheet_path = path.to_path_buf();
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
struct Rect {
    x: u32, y: u32,
    w: u32, h: u32,
}

#[derive(serde::Serialize)]
struct TextureData {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    nine_patch: Option<Rect>
}

struct ImageTexture {
    name: String,
    img: RgbaImage,
    nine_patch: Option<Rect>
}

impl ImageTexture {
    const fn new(name: String, img: RgbaImage, nine_patch: Option<Rect>) -> Self {
        ImageTexture {
            name, img, nine_patch
        }
    }
}

fn visit_dir(dir: PathBuf, collector: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if dir.is_dir() {
        let paths = std::fs::read_dir(dir)?;
        for path in paths {
            let path = path?.path();
            if path.is_dir() {
                visit_dir(path, collector)?;
            } else {
                collector.push(path);
            }
        }
    }
    Ok(())
}

fn get_extension_from_filename(filename: &Path) -> Option<&str> {
    filename
        .extension()
        .and_then(std::ffi::OsStr::to_str)
}

fn find_nine_patch_file(filename: &Path) -> Option<Rect> {
    let filename = filename.with_extension("json");
    let Ok(file) = std::fs::read_to_string(&filename) else { 
        let Ok(file) = std::fs::read_to_string(filename.with_extension("ron")) else { return None };
        let Ok(rect) = ron::from_str::<Rect>(&file) else { return None };
        return Some(rect);
    };
    let Ok(rect) = serde_json::from_str::<Rect>(&file) else { return None };
    Some(rect)
}

pub fn pack(config: PackerConfig) -> anyhow::Result<()> {
    let mut image_paths = vec![];

    for folder in config.folders.iter() {
        visit_dir(folder.to_path_buf(), &mut image_paths)?;
    }

    let images = image_paths.iter().filter_map(|file| {
        if get_extension_from_filename(file) != Some("png") {
            return None;
        }
        println!("Found {}", file.display());
        let nine_patch = if config.options.features.nine_patch {
            find_nine_patch_file(file)
        } else { None };
        let filename = if !config.options.show_extension {
            file.with_extension("").to_str().unwrap_or_default().to_owned()
        } else {
            file.to_str().unwrap_or_default().to_owned()
        };
        let Ok(img) = image::open(file) else { return None };
        Some(ImageTexture::new(filename, img.to_rgba8(), nine_patch))
    }).collect::<Vec<ImageTexture>>();

    let items = images.iter().map(|img| 
        Item::new(img, img.img.width() as usize, img.img.height() as usize, Rotation::None)
    ).collect::<Vec<Item<&ImageTexture>>>();

    if let Ok((w, h, packed)) = crunch::pack_into_po2(config.options.max_size, items) {
        let mut atlas_json = PackerAtlas::default();
        let mut atlas: RgbaImage = ImageBuffer::from_fn(
            w as u32, 
            h as u32, 
            |_, _| image::Rgba([0, 0, 0, 0])
        );

        // Pack all images
        for (rect, image_data) in packed {
            let (x, y) = (rect.x as u32, rect.y as u32);
            let (width, height) = (rect.w as u32, rect.h as u32);

            let view = image_data.img.view(0, 0, width, height);
            atlas.copy_from(&view, x, y)?;
            atlas_json.add(
                &image_data.name, x, y, rect.w as u32, rect.h as u32, image_data.nine_patch);
        }
        
        let mut path = config.output_path.clone();
        path.push(&config.name);
        path.set_extension("png");
        atlas_json.add_sheet_path(&path);

        let mut file_path = config.output_path.clone();
        file_path.push(&config.name);

        if !config.output_path.is_dir() {
            std::fs::create_dir_all(&config.output_path)?;
        }

        atlas.save(path)?;
        match config.output_type {
            OutputType::Json => save_output::<JsonOutput>(file_path, atlas_json)?,
            OutputType::Binary => save_output::<BinaryOutput>(file_path, atlas_json)?,
            OutputType::Ron => save_output::<RonOutput>(file_path, atlas_json)?,
            OutputType::Template => save_output_from(
                TemplateOutput(config), file_path, atlas_json
            )?
        }
        Ok(())
    } else {
        Err(PackerError::FailedToPacked)?
    }
}

#[derive(serde::Serialize)]
struct TemplateGlobals {
    atlas: PackerAtlas,
    config: PackerConfig
}
