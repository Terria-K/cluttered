use std::path::PathBuf;
use serde_json as json;

use binary_rw::{MemoryStream, BinaryWriter};

use crate::error::PackerError;

use super::{PackerAtlas, PackerConfig, TemplateGlobals};

pub(super) trait Output {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()>;
}

#[derive(Default)]
pub(super) struct JsonOutput;
#[derive(Default)]
pub(super) struct RonOutput;
#[derive(Default)]
pub(super) struct BinaryOutput;

pub(super) struct TemplateOutput(pub(super) PackerConfig);

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
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("json");
        let packer_atlas = json::to_string_pretty::<PackerAtlas>(&atlas)?.replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for RonOutput {
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("ron");
        let packer_atlas = ron::to_string::<PackerAtlas>(&atlas)?.replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for BinaryOutput {
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("bin");
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
