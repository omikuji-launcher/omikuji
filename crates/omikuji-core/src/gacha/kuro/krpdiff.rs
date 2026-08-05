use anyhow::{Result, anyhow, bail};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const CHUNK: usize = 256 * 1024;

#[derive(Clone)]
struct FileEntry {
    path: String,
    size: u64,
}

struct Cover {
    old_delta: i64,
    gap: u64,
    len: u64,
}

#[derive(Clone, Copy)]
struct Clip {
    offset: u64,
    size: u64,
    comp_size: u64,
}

pub struct Krpdiff {
    path: PathBuf,
    old_files: Vec<FileEntry>,
    new_files: Vec<FileEntry>,
    new_dirs: Vec<String>,
    new_empty_files: Vec<String>,
    old_ref_size: u64,
    new_ref_size: u64,
    covers_clip: Clip,
    cover_count: u64,
    rle_ctrl_clip: Clip,
    rle_code_clip: Clip,
    diff_clip: Clip,
}

impl Krpdiff {
    pub fn open(path: &Path) -> Result<Self> {
        let mut f = File::open(path)?;
        parse(&mut f, path.to_path_buf())
    }

    pub fn apply(
        &self,
        old_root: &Path,
        out_root: &Path,
        mut on_bytes: impl FnMut(u64),
    ) -> Result<()> {
        for fe in &self.old_files {
            let full = old_root.join(&fe.path);
            let actual = std::fs::metadata(&full)
                .map_err(|e| anyhow!("old file missing {}: {}", full.display(), e))?
                .len();
            if actual != fe.size {
                bail!(
                    "old file size mismatch {}: expected {}, got {}",
                    full.display(),
                    fe.size,
                    actual
                );
            }
        }

        for d in &self.new_dirs {
            std::fs::create_dir_all(out_root.join(d.trim_end_matches('/')))?;
        }
        for p in &self.new_empty_files {
            let full = out_root.join(p);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            File::create(&full)?;
        }

        let mut old = OldConcat::open(old_root, &self.old_files)?;
        let mut out = SeqWriter::new(out_root, &self.new_files);
        let covers = self.parse_covers()?;

        let mut diff = self.clip_reader(self.diff_clip)?;
        let mut rle = if self.rle_ctrl_clip.size > 0 {
            Some(RleState {
                ctrl: self.clip_reader(self.rle_ctrl_clip)?,
                code: self.clip_reader(self.rle_code_clip)?,
                set_len: 0,
                set_val: 0,
                copy_len: 0,
            })
        } else {
            None
        };

        let mut buf = vec![0u8; CHUNK];
        let mut read_pos: i64 = 0;
        for c in &covers {
            read_pos = read_pos.wrapping_add(c.old_delta);
            if self.old_ref_size > 0 {
                let sz = self.old_ref_size as i64;
                read_pos = read_pos.rem_euclid(sz);
            }
            if c.gap > 0 {
                pump(
                    &mut *diff,
                    &mut out,
                    &mut rle,
                    c.gap,
                    &mut buf,
                    &mut on_bytes,
                )?;
            }
            if c.len > 0 {
                old.seek_to(read_pos as u64);
                pump(&mut old, &mut out, &mut rle, c.len, &mut buf, &mut on_bytes)?;
            }
            read_pos = read_pos.wrapping_add(c.len as i64);
        }
        let written = out.written();
        if written < self.new_ref_size {
            pump(
                &mut *diff,
                &mut out,
                &mut rle,
                self.new_ref_size - written,
                &mut buf,
                &mut on_bytes,
            )?;
        }
        out.finish()?;
        if out.written() != self.new_ref_size {
            bail!(
                "patched output size mismatch: expected {}, wrote {}",
                self.new_ref_size,
                out.written()
            );
        }
        Ok(())
    }

    fn clip_reader(&self, clip: Clip) -> Result<Box<dyn Read>> {
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(clip.offset))?;
        if clip.comp_size == 0 {
            return Ok(Box::new(f.take(clip.size)));
        }
        let mut dec = zstd::stream::read::Decoder::new(f.take(clip.comp_size))?;
        dec.set_parameter(zstd::zstd_safe::DParameter::WindowLogMax(31))?;
        Ok(Box::new(dec))
    }

    fn parse_covers(&self) -> Result<Vec<Cover>> {
        let mut r = self.clip_reader(self.covers_clip)?;
        let mut covers = Vec::with_capacity(self.cover_count as usize);
        for _ in 0..self.cover_count {
            let first = read_u8(&mut r)?;
            let negative = first >> 7 != 0;
            let abs = read_packed(&mut r, 1, Some(first))? as i64;
            covers.push(Cover {
                old_delta: if negative { -abs } else { abs },
                gap: read_packed(&mut r, 0, None)?,
                len: read_packed(&mut r, 0, None)?,
            });
        }
        Ok(covers)
    }
}

fn pump(
    src: &mut dyn Read,
    out: &mut SeqWriter,
    rle: &mut Option<RleState>,
    mut n: u64,
    buf: &mut [u8],
    on_bytes: &mut impl FnMut(u64),
) -> Result<()> {
    while n > 0 {
        let step = (buf.len() as u64).min(n) as usize;
        src.read_exact(&mut buf[..step])?;
        if let Some(rle) = rle {
            rle.add_over(&mut buf[..step])?;
        }
        out.write_all(&buf[..step])?;
        on_bytes(step as u64);
        n -= step as u64;
    }
    Ok(())
}

struct RleState {
    ctrl: Box<dyn Read>,
    code: Box<dyn Read>,
    set_len: u64,
    set_val: u8,
    copy_len: u64,
}

impl RleState {
    fn add_over(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut i = 0usize;
        let mut code_buf = [0u8; 4096];
        while i < buf.len() {
            if self.set_len == 0 && self.copy_len == 0 {
                let first = read_u8(&mut self.ctrl)?;
                let kind = first >> 6;
                let len = read_packed(&mut self.ctrl, 2, Some(first))? + 1;
                match kind {
                    3 => self.copy_len = len,
                    2 => {
                        self.set_val = read_u8(&mut self.code)?;
                        self.set_len = len;
                    }
                    k => {
                        self.set_val = 0u8.wrapping_sub(k);
                        self.set_len = len;
                    }
                }
            }
            if self.set_len > 0 {
                let step = (self.set_len.min((buf.len() - i) as u64)) as usize;
                if self.set_val != 0 {
                    for b in &mut buf[i..i + step] {
                        *b = b.wrapping_add(self.set_val);
                    }
                }
                self.set_len -= step as u64;
                i += step;
            } else {
                let step =
                    (self.copy_len.min((buf.len() - i) as u64)).min(code_buf.len() as u64) as usize;
                self.code.read_exact(&mut code_buf[..step])?;
                for (b, c) in buf[i..i + step].iter_mut().zip(&code_buf[..step]) {
                    *b = b.wrapping_add(*c);
                }
                self.copy_len -= step as u64;
                i += step;
            }
        }
        Ok(())
    }
}

struct OldConcat {
    segments: Vec<(File, u64, u64)>,
    pos: u64,
    idx: usize,
}

impl OldConcat {
    fn open(root: &Path, files: &[FileEntry]) -> Result<Self> {
        let mut segments = Vec::with_capacity(files.len());
        let mut start = 0u64;
        for fe in files {
            segments.push((File::open(root.join(&fe.path))?, start, fe.size));
            start += fe.size;
        }
        Ok(Self {
            segments,
            pos: 0,
            idx: 0,
        })
    }

    fn seek_to(&mut self, pos: u64) {
        self.pos = pos;
        while self.idx > 0 && pos < self.segments[self.idx].1 {
            self.idx -= 1;
        }
        while self.idx + 1 < self.segments.len() {
            let (_, start, len) = self.segments[self.idx];
            if pos >= start + len {
                self.idx += 1;
            } else {
                break;
            }
        }
    }
}

impl Read for OldConcat {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let Some((file, start, len)) = self.segments.get_mut(self.idx) else {
                return Ok(0);
            };
            let in_seg = self.pos - *start;
            if in_seg >= *len {
                if self.idx + 1 < self.segments.len() {
                    self.idx += 1;
                    continue;
                }
                return Ok(0);
            }
            file.seek(SeekFrom::Start(in_seg))?;
            let avail = (*len - in_seg).min(buf.len() as u64) as usize;
            let n = file.read(&mut buf[..avail])?;
            self.pos += n as u64;
            return Ok(n);
        }
    }
}

struct SeqWriter {
    root: PathBuf,
    files: Vec<FileEntry>,
    next: usize,
    current: Option<(File, u64)>,
    written: u64,
}

impl SeqWriter {
    fn new(root: &Path, files: &[FileEntry]) -> Self {
        Self {
            root: root.to_path_buf(),
            files: files.to_vec(),
            next: 0,
            current: None,
            written: 0,
        }
    }

    fn written(&self) -> u64 {
        self.written
    }

    fn write_all(&mut self, mut data: &[u8]) -> Result<()> {
        while !data.is_empty() {
            if self.current.is_none() {
                let Some(fe) = self.files.get(self.next) else {
                    bail!("patched output overflows the new file list");
                };
                self.next += 1;
                let full = self.root.join(&fe.path);
                if let Some(parent) = full.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let file = File::create(&full)?;
                if fe.size == 0 {
                    continue;
                }
                self.current = Some((file, fe.size));
            }
            let (file, remaining) = self.current.as_mut().unwrap();
            let step = (*remaining).min(data.len() as u64) as usize;
            file.write_all(&data[..step])?;
            *remaining -= step as u64;
            self.written += step as u64;
            data = &data[step..];
            if *remaining == 0 {
                self.current = None;
            }
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<()> {
        while self.next < self.files.len() {
            let fe = &self.files[self.next];
            self.next += 1;
            if fe.size != 0 {
                bail!("patched output ended before {} was produced", fe.path);
            }
            let full = self.root.join(&fe.path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            File::create(&full)?;
        }
        Ok(())
    }
}

fn parse(f: &mut File, path: PathBuf) -> Result<Krpdiff> {
    let magic = read_until(f, b'&', 16)?;
    if magic != "HDIFF19" {
        bail!("not a krpdiff: magic {:?}", magic);
    }
    let comp = read_until(f, b'&', 16)?;
    if !comp.is_empty() && comp != "zstd" {
        bail!("unsupported compression {:?}", comp);
    }
    let _checksum = read_until(f, b'\0', 16)?;
    let old_is_dir = read_u8(f)? != 0;
    let new_is_dir = read_u8(f)? != 0;
    if !old_is_dir || !new_is_dir {
        bail!("not a directory diff");
    }

    let old_path_count = read_packed(f, 0, None)?;
    let _old_path_sum = read_packed(f, 0, None)?;
    let new_path_count = read_packed(f, 0, None)?;
    let _new_path_sum = read_packed(f, 0, None)?;
    let old_ref_count = read_packed(f, 0, None)?;
    let old_ref_size = read_packed(f, 0, None)?;
    let new_ref_count = read_packed(f, 0, None)?;
    let new_ref_size = read_packed(f, 0, None)?;
    let same_pair_count = read_packed(f, 0, None)?;
    let _same_size = read_packed(f, 0, None)?;
    let execute_count = read_packed(f, 0, None)?;
    let _private_reserved = read_packed(f, 0, None)?;
    let private_extern_size = read_packed(f, 0, None)?;
    let extern_size = read_packed(f, 0, None)?;
    let head_size = read_packed(f, 0, None)?;
    let head_comp_size = read_packed(f, 0, None)?;
    let checksum_byte_size = read_packed(f, 0, None)?;
    if same_pair_count != 0 || execute_count != 0 {
        bail!(
            "unsupported krpdiff features: samePairs={}, executes={}",
            same_pair_count,
            execute_count
        );
    }

    f.seek(SeekFrom::Current(checksum_byte_size as i64 * 4))?;

    let head_start = f.stream_position()?;
    let head_bytes = if head_comp_size > 0 {
        head_comp_size
    } else {
        head_size
    };
    let head = if head_comp_size > 0 {
        let mut dec = zstd::stream::read::Decoder::new(Read::by_ref(f).take(head_comp_size))?;
        dec.set_parameter(zstd::zstd_safe::DParameter::WindowLogMax(31))?;
        parse_head(
            &mut dec,
            old_path_count,
            new_path_count,
            old_ref_count,
            new_ref_count,
        )?
    } else {
        parse_head(
            &mut Read::by_ref(f).take(head_size),
            old_path_count,
            new_path_count,
            old_ref_count,
            new_ref_count,
        )?
    };
    f.seek(SeekFrom::Start(
        head_start + head_bytes + private_extern_size + extern_size,
    ))?;

    let magic13 = read_until(f, b'&', 16)?;
    if magic13 != "HDIFF13" {
        bail!("expected inner HDIFF13, got {:?}", magic13);
    }
    let comp13 = read_until(f, b'\0', 16)?;
    if !comp13.is_empty() && comp13 != "zstd" {
        bail!("unsupported inner compression {:?}", comp13);
    }

    let _new_data_size = read_packed(f, 0, None)?;
    let _old_data_size = read_packed(f, 0, None)?;
    let cover_count = read_packed(f, 0, None)?;
    let cover_size = read_packed(f, 0, None)?;
    let cover_comp_size = read_packed(f, 0, None)?;
    let ctrl_size = read_packed(f, 0, None)?;
    let ctrl_comp_size = read_packed(f, 0, None)?;
    let code_size = read_packed(f, 0, None)?;
    let code_comp_size = read_packed(f, 0, None)?;
    let diff_size = read_packed(f, 0, None)?;
    let diff_comp_size = read_packed(f, 0, None)?;

    let stored = |size: u64, comp: u64| if comp > 0 { comp } else { size };
    let covers_off = f.stream_position()?;
    let ctrl_off = covers_off + stored(cover_size, cover_comp_size);
    let code_off = ctrl_off + stored(ctrl_size, ctrl_comp_size);
    let diff_off = code_off + stored(code_size, code_comp_size);

    Ok(Krpdiff {
        path,
        old_files: head.old_files,
        new_files: head.new_files,
        new_dirs: head.new_dirs,
        new_empty_files: head.new_empty_files,
        old_ref_size,
        new_ref_size,
        covers_clip: Clip {
            offset: covers_off,
            size: cover_size,
            comp_size: cover_comp_size,
        },
        cover_count,
        rle_ctrl_clip: Clip {
            offset: ctrl_off,
            size: ctrl_size,
            comp_size: ctrl_comp_size,
        },
        rle_code_clip: Clip {
            offset: code_off,
            size: code_size,
            comp_size: code_comp_size,
        },
        diff_clip: Clip {
            offset: diff_off,
            size: diff_size,
            comp_size: diff_comp_size,
        },
    })
}

struct Head {
    old_files: Vec<FileEntry>,
    new_files: Vec<FileEntry>,
    new_dirs: Vec<String>,
    new_empty_files: Vec<String>,
}

fn parse_head(
    r: &mut impl Read,
    old_path_count: u64,
    new_path_count: u64,
    old_ref_count: u64,
    new_ref_count: u64,
) -> Result<Head> {
    let mut old_paths = Vec::with_capacity(old_path_count as usize);
    for _ in 0..old_path_count {
        old_paths.push(read_until(r, b'\0', 4096)?);
    }
    let mut new_paths = Vec::with_capacity(new_path_count as usize);
    for _ in 0..new_path_count {
        new_paths.push(read_until(r, b'\0', 4096)?);
    }

    let old_ref_idx = read_incremental_indexes(r, old_ref_count)?;
    let new_ref_idx = read_incremental_indexes(r, new_ref_count)?;
    let mut old_sizes = Vec::with_capacity(old_ref_count as usize);
    for _ in 0..old_ref_count {
        old_sizes.push(read_packed(r, 0, None)?);
    }
    let mut new_sizes = Vec::with_capacity(new_ref_count as usize);
    for _ in 0..new_ref_count {
        new_sizes.push(read_packed(r, 0, None)?);
    }
    for _ in 0..new_ref_count {
        read_packed(r, 0, None)?;
    }

    let (old_files, _, _) = split_paths(&old_paths, &old_ref_idx, &old_sizes);
    let (new_files, new_dirs, new_empty_files) = split_paths(&new_paths, &new_ref_idx, &new_sizes);
    Ok(Head {
        old_files,
        new_files,
        new_dirs,
        new_empty_files,
    })
}

fn split_paths(
    paths: &[String],
    ref_idx: &[u64],
    sizes: &[u64],
) -> (Vec<FileEntry>, Vec<String>, Vec<String>) {
    let mut files = Vec::with_capacity(ref_idx.len());
    let mut dirs = Vec::new();
    let mut empty_files = Vec::new();
    let mut next_ref = 0usize;
    for (i, path) in paths.iter().enumerate() {
        if next_ref < ref_idx.len() && i as u64 == ref_idx[next_ref] {
            files.push(FileEntry {
                path: path.clone(),
                size: sizes[next_ref],
            });
            next_ref += 1;
        } else if path.is_empty() || path.ends_with('/') {
            dirs.push(path.clone());
        } else {
            empty_files.push(path.clone());
        }
    }
    (files, dirs, empty_files)
}

fn read_incremental_indexes(r: &mut impl Read, count: u64) -> Result<Vec<u64>> {
    let mut out = Vec::with_capacity(count as usize);
    let mut back = -1i64;
    for _ in 0..count {
        back += 1 + read_packed(r, 0, None)? as i64;
        out.push(back as u64);
    }
    Ok(out)
}

fn read_u8(r: &mut impl Read) -> Result<u8> {
    let mut b = [0u8; 1];
    r.read_exact(&mut b)?;
    Ok(b[0])
}

fn read_until(r: &mut impl Read, delim: u8, limit: usize) -> Result<String> {
    let mut buf = Vec::with_capacity(16);
    loop {
        let b = read_u8(r)?;
        if b == delim {
            break;
        }
        buf.push(b);
        if buf.len() > limit {
            bail!("delimited string exceeds {} bytes", limit);
        }
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn read_packed(r: &mut impl Read, tag_bits: u8, first: Option<u8>) -> Result<u64> {
    let code = match first {
        Some(b) => b,
        None => read_u8(r)?,
    };
    let mask = (1u8 << (7 - tag_bits)) - 1;
    let mut value = (code & mask) as u64;
    if code & (1 << (7 - tag_bits)) == 0 {
        return Ok(value);
    }
    loop {
        if value >> 57 != 0 {
            bail!("packed varint overflow");
        }
        let b = read_u8(r)?;
        value = (value << 7) | (b & 0x7f) as u64;
        if b & 0x80 == 0 {
            return Ok(value);
        }
    }
}
