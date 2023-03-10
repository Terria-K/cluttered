use std::path::PathBuf;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde_json as json;

use binary_rw::{MemoryStream, BinaryWriter};

use crate::error::PackerError;

use super::{PackerAtlas, Config, TemplateGlobals};

pub(super) trait Output {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()>;
}

#[derive(Default)]
pub(super) struct JsonOutput;
#[derive(Default)]
pub(super) struct RonOutput;
#[derive(Default)]
pub(super) struct TomlOutput;

pub(super) struct BinaryOutput(pub(super) Config);

pub(super) struct TemplateOutput(pub(super) Config);

impl Output for TemplateOutput {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        let Some(ref template_path) = self.0.template_path else { 
            Err(PackerError::NoTemplateFile)?
        };
        let template = std::fs::read_to_string(template_path)?;
        let mut handlerbars = handlebars::Handlebars::new();
        handlerbars.set_strict_mode(true);
        handlerbars.register_template_string("t1", template)?;
        let extension = template_path
            .extension()
            .unwrap_or_else(|| std::ffi::OsStr::new(""));
        let template_path = path.with_extension(extension);
        let globals = TemplateGlobals {
            atlas,
            config: self.0.clone()
        };

        let compiled = handlerbars.render("t1", &globals)?.replace('\\', "/");
        std::fs::write(template_path, compiled)?;
        Ok(())
    }
}

impl Output for JsonOutput {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        let path = path.with_extension("json");
        let packer_atlas = json::to_string_pretty::<PackerAtlas>(&atlas)?.replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for RonOutput {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        let path = path.with_extension("ron");
        let packer_atlas = to_string_pretty::<PackerAtlas>(&atlas, PrettyConfig::default())?
            .replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for TomlOutput {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        let path = path.with_extension("toml");
        let packer_atlas = toml::to_string_pretty::<PackerAtlas>(&atlas)?
            .replace("\\\\", "/")
            .replace('\\', "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for BinaryOutput {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        let path = path.with_extension("bin");
        let mut fs = MemoryStream::new();
        let mut writer = binary_rw::BinaryWriter::new(&mut fs, binary_rw::Endian::Little);
        let sheet_path = atlas.sheet_path.to_str().unwrap_or_default().replace('\\', "/");
        write_sharp_string(&mut writer, sheet_path)?;
        let length = atlas.frames.len();
        writer.write_u32(length as u32)?;
        for (frame_key, data) in atlas.frames {
            let frame_key = frame_key.replace('\\', "/");
            write_sharp_string(&mut writer, frame_key)?;
            writer.write_u32(data.x)?;
            writer.write_u32(data.y)?;
            writer.write_u32(data.width)?;
            writer.write_u32(data.height)?;
            if !self.0.features.nine_patch {
                continue;
            }
            writer.write_bool(data.nine_patch.is_some())?;
            if let Some(nine_patch) = data.nine_patch {
                writer.write_u32(nine_patch.x)?;
                writer.write_u32(nine_patch.y)?;
                writer.write_u32(nine_patch.w)?;
                writer.write_u32(nine_patch.h)?;
            }
        }

        let buffer: Vec<u8> = fs.into();
        std::fs::write(path, buffer)?;
        Ok(())
    }
}

fn write_sharp_string<S>(writer: &mut BinaryWriter, value: S) -> anyhow::Result<()>
where S: AsRef<str> {
    let bytes = value.as_ref().as_bytes();
    writer.write_u8(bytes.len() as u8)?;
    writer.write_bytes(bytes)?;
    Ok(())
}

pub(super) fn save_output<T>(path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()>
where T: Default + Output {
    let output = T::default();
    output.out(path, atlas)
}

pub(super) fn save_output_from<T>(
    output: T, 
    path: PathBuf, 
    atlas: PackerAtlas
) -> anyhow::Result<()>
where T: Output {
    output.out(path, atlas)
}
