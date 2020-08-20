use std::convert::{AsRef, TryInto};
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind, Write}; 
use std::path::{Path, PathBuf};

use memmap::{MmapMut, MmapOptions};
use rand::{rngs::OsRng, Rng};

const NODE_SIZE: u64 = 32;

pub fn new_seed() -> [u8; 32] {
    OsRng.gen()
}

pub fn init_output_dir<P: AsRef<Path> + Copy>(path: P, overwrite: bool) -> io::Result<()> {
    if path.as_ref().exists() {
        match overwrite {
            true => {
                fs::remove_dir_all(path)?;
            },
            false => {
                return Err(io::Error::new(ErrorKind::AlreadyExists, "target existed"))
            }
        }
    }

    fs::create_dir_all(path)
}

pub fn output_file_name<P: AsRef<Path>>(src_file: P, output_dir: P, ext: &str) -> io::Result<PathBuf> {
    let name = match src_file.as_ref().file_name() {
        None => {
            return Err(io::Error::new(ErrorKind::InvalidInput, "malformed input filename"))
        },
        Some(s) => s,
    };

    let mut res = PathBuf::from(output_dir.as_ref());
    res.push(name);
    res.set_extension(OsStr::new(ext));

    Ok(res)
}

pub fn count_nodes<P: AsRef<Path>>(path: P) -> io::Result<usize> {
    let file_info = fs::metadata(path)?;

    match (file_info.len() / NODE_SIZE).try_into() {
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
    file.write_all(data)?;

    Ok(())
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

#[cfg(test)]
pub(crate) mod test {
    use std::fs;
    use std::io;
    use std::ops::Deref;
    use std::path::Path;

    use rand;
    use storage_proofs::hasher::{Domain, Hasher, PedersenHasher};

    use super::*;

    // helper: create a sample data file
    pub fn gen_sample_file<H: 'static>(nodes: usize, path: &Path) -> io::Result<usize> 
    where
        H: Hasher,
    { 
        let rng = &mut rand::thread_rng();
        let data: Vec<u8> = (0..nodes)
            .flat_map(|_| {
                let v: H::Domain = H::Domain::random(rng);
                v.into_bytes()
            })
            .collect();
        
        write_file(path, &data).expect("error saving temporary sample data");

        Ok(data.len())
    }

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

        init_output_dir(output_dir, true)
            .expect("cannot setup test sample dir");
        gen_sample_file::<PedersenHasher>(128, &sample_file)
            .expect("cannot setup test sample file");

        let example1 = output_file_name(sample_file, PathBuf::from("./another_example"), "txt")
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

    // this is actually used as an test sample for benchmark
    // TODO: move this logic to integrate test logic later
    #[test]
    #[ignore]
    fn gen_one_giga_bytes_sample() {
        let sample_dir = Path::new("./sample");
        init_output_dir(sample_dir, true).expect("failed to setting the sample dir");

        let input_size: usize = 1024 * 1024 * 1024;
        let input_path = sample_dir.join("sample.dat");
        
        gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();
    }
}