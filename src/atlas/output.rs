use std::path::PathBuf;

use binary_rw::MemoryStream;

use super::PackerAtlas;

pub(super) trait Output {
    fn out(&self, path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()>;
}

#[derive(Default)]
pub(super) struct JsonOutput;
#[derive(Default)]
pub(super) struct RonOutput;
#[derive(Default)]
pub(super) struct BinaryOutput;

impl Output for JsonOutput {
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("json");
        let packer_atlas = serde_json::to_string_pretty::<PackerAtlas>(&atlas)?;
        let packer_atlas = packer_atlas.replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for RonOutput {
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("ron");
        let packer_atlas = ron::to_string::<PackerAtlas>(&atlas)?;
        let packer_atlas = packer_atlas.replace("\\\\", "/");
        std::fs::write(path, packer_atlas)?;
        Ok(())
    }
}

impl Output for BinaryOutput {
    fn out(&self, mut path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()> {
        path.set_extension("bin");
        let mut fs = MemoryStream::new();
        let mut writer = binary_rw::BinaryWriter::new(&mut fs, binary_rw::Endian::Little);
        writer.write_string(atlas.sheet_path.to_str().unwrap_or_default())?;
        let length = atlas.frames.len();
        writer.write_u32(length as u32)?;
        for (frame_key, data) in atlas.frames {
            let frame_key = frame_key.replace('\\', "/");
            writer.write_string(frame_key)?;
            writer.write_u32(data.x)?;
            writer.write_u32(data.y)?;
            writer.write_u32(data.width)?;
            writer.write_u32(data.height)?;
            writer.write_bool(data.rotated)?;
        }

        let buffer: Vec<u8> = fs.into();
        std::fs::write(path, buffer)?;
        Ok(())
    }
}

pub(super) fn save_output<T>(path: PathBuf, atlas: PackerAtlas) -> anyhow::Result<()>
where T: Default + Output{
    let output = T::default();
    output.out(path, atlas)
}
