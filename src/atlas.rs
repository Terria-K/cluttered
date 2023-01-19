extern crate binary_rw;
use std::{path::{PathBuf, Path}, collections::HashMap};

use binary_rw::MemoryStream;
use crunch::{Item, Rotation};
use image::{RgbaImage, ImageBuffer, GenericImage, GenericImageView};

#[derive(serde::Deserialize)]
pub struct PackerConfig {
    pub name: String,
    pub output_path: PathBuf,
    pub output_type: OutputType,
    pub folders: Vec<PathBuf>,
    pub options: PackerConfigOptions
}

#[derive(serde::Deserialize, clap::ValueEnum, Clone)]
pub enum OutputType {
    Json,
    Binary,
    Ron
}

#[derive(Default, serde::Serialize)]
struct PackerAtlas {
    sheet_path: PathBuf,
    frames: HashMap<String, TextureData>
}

impl PackerAtlas {
    fn add(&mut self, name: &str, x: u32, y: u32, width: u32, height: u32) {
        self.frames.insert(name.into(), TextureData {
            x, y, width, height
        });
    }

    fn add_sheet_path(&mut self, path: &Path) {
        self.sheet_path = path.to_path_buf();
    }
}

#[derive(serde::Serialize)]
struct TextureData {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct ImageTexture {
    name: String,
    img: RgbaImage
}

impl ImageTexture {
    const fn new(name: String, img: RgbaImage) -> Self {
        ImageTexture {
            name, img
        }
    }
}

#[derive(serde::Deserialize)]
pub struct PackerConfigOptions {
    max_size: usize,
    show_extension: bool,
    rotation: bool
}

impl Default for PackerConfigOptions {
    fn default() -> Self {
        PackerConfigOptions { 
            max_size: 1024,
            show_extension: true,
            rotation: false
        }
    }
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

pub fn pack(config: PackerConfig) -> anyhow::Result<()> {
    let mut image_paths = Vec::new();

    for folder in config.folders {
        let mut paths = Vec::new();
        visit_dir(folder, &mut paths)?;
        image_paths.extend(paths);
    }

    let images: Vec<ImageTexture> = image_paths.iter().filter_map(|file| {
        let filename = file;
        let ext = get_extension_from_filename(filename);
        if ext != Some("png") {
            return None;
        }
        println!("Found {}",filename.display());
        let filename = if !config.options.show_extension {
            filename.with_extension("")
                .to_str().unwrap_or_default().to_owned()
        } else {
            filename.to_str().unwrap_or_default().to_owned()
        };
        let img = image::open(file).unwrap().to_rgba8();
        Some(ImageTexture::new(filename, img))
    }).collect();

    let items: Vec<Item<&ImageTexture>> = images.iter().enumerate().map(|(_, img)| {
        let rotation = match config.options.rotation {
            true => Rotation::Allowed,
            false => Rotation::None
        };
        Item::new(img, img.img.width() as usize, img.img.height() as usize, rotation)
    }).collect();

    if let Ok((w, h, packed)) = crunch::pack_into_po2(config.options.max_size, items) {
        let mut atlas_json = PackerAtlas::default();
        let mut atlas: RgbaImage = ImageBuffer::from_fn(
            w as u32, 
            h as u32, 
            |_, _| image::Rgba([0, 0, 0, 0])
        );

        // Pack all images
        for (rect, img) in &packed {
            let (x, y) = (rect.x as u32, rect.y as u32);
            let (width, height) = (img.img.width(), img.img.height());
            let view = img.img.view(0, 0, width, height);
            atlas.copy_from(&view, x, y)?;
            atlas_json.add(&img.name, x, y, rect.w as u32, rect.h as u32);
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
            OutputType::Json => {
                file_path.set_extension("json");
                let packer_atlas = serde_json::to_string_pretty::<PackerAtlas>(&atlas_json)?;
                let packer_atlas = packer_atlas.replace("\\\\", "/");
                std::fs::write(file_path, packer_atlas)?;
            },
            OutputType::Binary => {
                file_path.set_extension("bin");
                let mut fs = MemoryStream::new();
                let mut writer = binary_rw::BinaryWriter::new(&mut fs, binary_rw::Endian::Little);
                writer.write_string(atlas_json.sheet_path.to_str().unwrap_or_default())?;
                let length = atlas_json.frames.len();
                writer.write_u32(length as u32)?;
                for (frame_key, data) in atlas_json.frames {
                    let frame_key = frame_key.replace('\\', "/");
                    writer.write_string(frame_key)?;
                    writer.write_u32(data.x)?;
                    writer.write_u32(data.y)?;
                    writer.write_u32(data.width)?;
                    writer.write_u32(data.height)?;
                }

                let buffer: Vec<u8> = fs.into();
                std::fs::write(file_path, buffer)?;
            },
            OutputType::Ron => {
                file_path.set_extension("ron");
                let packer_atlas = ron::to_string::<PackerAtlas>(&atlas_json)?;
                let packer_atlas = packer_atlas.replace("\\\\", "/");
                std::fs::write(file_path, packer_atlas)?;
            },
        }

        Ok(())
    } else {
        panic!("failed to packed images")
    }
}

