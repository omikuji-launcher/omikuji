use anyhow::Result;
use flate2::read::GzDecoder;
use std::io::{Read, Seek};
use std::path::Path;
use xz2::read::XzDecoder;
use zip::ZipArchive;
use zstd::stream::read::Decoder as ZstdDecoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    TarGz,
    TarXz,
    TarZst,
    Tar,
    Zip,
}

const EXTENSIONS: &[(&str, ArchiveKind)] = &[
    (".tar.gz", ArchiveKind::TarGz),
    (".tgz", ArchiveKind::TarGz),
    (".tar.xz", ArchiveKind::TarXz),
    (".tar.zst", ArchiveKind::TarZst),
    (".tar", ArchiveKind::Tar),
    (".zip", ArchiveKind::Zip),
];

fn split(name: &str) -> Option<(&str, ArchiveKind)> {
    let lower = name.to_ascii_lowercase();
    EXTENSIONS
        .iter()
        .find(|(ext, _)| lower.ends_with(ext))
        .map(|(ext, kind)| (&name[..name.len() - ext.len()], *kind))
}

pub fn stem(name: &str) -> &str {
    split(name).map_or(name, |(stem, _)| stem)
}

impl ArchiveKind {
    pub fn from_name(name: &str) -> Option<Self> {
        split(name).map(|(_, kind)| kind)
    }

    pub fn unpack(self, reader: impl Read + Seek, dest: &Path) -> Result<()> {
        match self {
            Self::TarGz => unpack_tar(GzDecoder::new(reader), dest),
            Self::TarXz => unpack_tar(XzDecoder::new(reader), dest),
            Self::TarZst => unpack_tar(ZstdDecoder::new(reader)?, dest),
            Self::Tar => unpack_tar(reader, dest),
            Self::Zip => Ok(ZipArchive::new(reader)?.extract(dest)?),
        }
    }
}

fn unpack_tar(reader: impl Read, dest: &Path) -> Result<()> {
    Ok(tar::Archive::new(reader).unpack(dest)?)
}
