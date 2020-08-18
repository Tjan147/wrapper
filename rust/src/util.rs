use std::fs::OpenOptions;
use std::io::{self, Write}; 
use std::path::Path;

use memmap::{MmapMut, MmapOptions};
use rand::{rngs::OsRng, Rng};

pub fn new_seed() -> [u8; 32] {
    OsRng.gen()
}

pub fn read_file_as_mmap(path: &Path) -> Result<MmapMut, io::Error> {
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

pub fn write_file(path: &Path, data: &[u8]) -> Result<(), io::Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    file.write_all(data)?;

    Ok(())
}

pub fn write_file_and_mmap(path: &Path, data: &[u8]) -> Result<MmapMut, io::Error> {
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
pub mod test {
    use std::fs;
    use std::io;
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

    // this is actually used as an test sample
    #[test]
    #[ignore]
    fn gen_one_giga_bytes_sample() {
        let sample_dir = Path::new("./sample");
        fs::create_dir(sample_dir).unwrap();

        let input_size: usize = 1024 * 1024 * 1024;
        let input_path = sample_dir.join("sample.txt");
        
        gen_sample_file::<PedersenHasher>(input_size / 32, input_path.as_path()).unwrap();
    }
}