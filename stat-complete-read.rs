use std::{
    // string::String,
    io,
    io::Read,
    fs::File,
};

fn main () -> Result<(), io::Error> {
    for _ in 0..100000 {
        let mut f = File::open("/proc/stat")?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        // print!("{}", String::from_utf8_lossy(&buf));
    }
    Ok(())
}
