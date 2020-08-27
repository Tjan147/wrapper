use std::convert::{AsRef, TryInto};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write}; 
use std::path::{Path, PathBuf};

use memmap::{MmapMut, MmapOptions};
use rand::{rngs::OsRng, Rng};
use storage_proofs::hasher::{Hasher, Domain};

const DEFAULT_NODE_SIZE: u64 = 32;
const DEFAULT_PARAMS_DIR: &str = "params";

pub fn rand_bytes() -> [u8; 32] {
    OsRng.gen() 
}

pub fn mk_default_layout<P: AsRef<Path> + Copy>(dir: P) -> io::Result<()> {
    fs::create_dir_all(dir.as_ref().join(DEFAULT_PARAMS_DIR))
}

pub fn init_output_dir<P: AsRef<Path> + Copy>(dir: P, clear_first: bool) -> io::Result<()> {
    if clear_first && dir.as_ref().exists() {
        fs::remove_dir_all(dir)?;
    }

    fs::create_dir_all(dir)?;
    mk_default_layout(dir)
}

pub fn target_file_name<S: AsRef<OsStr>>(src_file_path: &Path, output_dir: &Path, ext: S) -> io::Result<PathBuf> {
    let name = src_file_path
        .file_name()
        .ok_or_else(|| { io::Error::new(ErrorKind::InvalidInput, "cannot extract source file's name") })?;

    let mut res = PathBuf::new();
    res.push(output_dir);
    res.push(name);
    res.set_extension(ext);

    Ok(res)
}

pub fn target_param_file_name<S: AsRef<OsStr>>(replica_path: &Path, ext: S) -> io::Result<PathBuf> {
    let dir = replica_path
        .parent()
        .ok_or_else(|| { io::Error::new(ErrorKind::InvalidInput, "cannot extract target folder's path") })?;

    let mut param_dir = PathBuf::new();
    param_dir.push(dir);
    param_dir.push(DEFAULT_PARAMS_DIR);
    
    target_file_name(&replica_path, &param_dir, ext)
}

pub fn count_nodes<P: AsRef<Path>>(path: P) -> io::Result<usize> {
    let file_info = fs::metadata(path)?;

    match (file_info.len() / DEFAULT_NODE_SIZE).try_into() {
        Err(e) => {
            return Err(io::Error::new(ErrorKind::Other, e))
        },
        Ok(p) => Ok(p),
    }
}

pub fn read_file_as_mmap(path: &Path) -> io::Result<MmapMut> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    unsafe {
        MmapOptions::new()
            .map_mut(&file)
    }
}

pub fn write_file(path: &Path, data: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    file.write_all(data)
}

pub fn write_file_and_mmap(path: &Path, data: &[u8]) -> io::Result<MmapMut> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    file.write_all(data)?;

    unsafe {
        MmapOptions::new()
            .map_mut(&file)
    }
}

pub fn gen_sample_file<H: Hasher>(nodes: usize, path: &Path) -> io::Result<()> { 
    let rng = &mut rand::thread_rng();
    let data: Vec<u8> = (0..nodes)
        .flat_map(|_| {
            let v: H::Domain = H::Domain::random(rng);
            v.into_bytes()
        })
        .collect();
    
    write_file(path, &data)
}

#[cfg(test)]
pub(crate) mod test {
    use std::fs;
    use std::ops::Deref;
    use std::path::Path;

    use rand;
    use storage_proofs::hasher::PedersenHasher;

    use super::*;

    #[test]
    fn test_dir_init() {
        let output_dir = Path::new("./sample");

        // 1st
        init_output_dir(&output_dir, true)
            .expect("error 1st time create the output dir");
        // 2nd
        init_output_dir(&output_dir, false)
            .expect_err("2nd time create the output dir should fail");
        // 3rd
        let output_file = output_dir.join("sample.dat");
        gen_sample_file::<PedersenHasher>(1024, &output_file)
            .expect("error create a sample file in test output dir");
        init_output_dir(&output_dir, true).expect("error 3rd time create the output dir");
        assert_eq!(false, output_file.exists());
    }

    #[test]
    fn test_output_file_name() {
        let output_dir = Path::new("./sample");
        let sample_file = output_dir.join("sample.dat");

        let example1 = target_file_name(&sample_file, &PathBuf::from("./another_example"), "txt")
            .expect("error create the output filename");
        assert_eq!(Path::new("./another_example/sample.txt"), example1);
        let example2 = example1.with_extension("store_conf");
        assert_eq!(Path::new("./another_example/sample.store_conf"), example2);
    }

    #[test]
    fn test_file_backed_mmap() {
        let content = format!("the new generated random is {}.", rand::thread_rng().gen_range(1, 114514));
        let gen_path = Path::new("sample.dat");

        {
            let gen_map = write_file_and_mmap(gen_path, content.as_bytes())
                .expect("failed to write sample.dat");
            assert_eq!(content.as_bytes(), gen_map.deref());
        }

        let load_map = read_file_as_mmap(gen_path).expect("failed to read sample.dat");
        assert_eq!(content.as_bytes(), load_map.deref());

        fs::remove_file(gen_path).expect("failed to delete the sample.dat");
    }
}