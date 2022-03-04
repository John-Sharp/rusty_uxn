use std::io::Cursor;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;

fn main() {

    let mut c = Cursor::new(Vec::new()); 

    c.write(&[1,2,3,4]).unwrap();

    c.seek(SeekFrom::Start(1)).unwrap();
    c.write(&[9,]).unwrap();

    let v = c.into_inner();

    println!("{:?}",v);
}
