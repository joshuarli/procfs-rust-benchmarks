use std::{
    // string::String,
    io,
    io::Read,
    fs::File,
};

fn main () -> Result<(), io::Error> {
    for _ in 0..100000 {
        let mut f = File::open("/proc/1/statm")?;
        let mut buf = [0; 4096];
        f.read(&mut buf).unwrap();
        // print!("{}", String::from_utf8_lossy(&buf));
    }
    Ok(())
}
