extern crate termion;
use std::io::{Read, stdout, Write};
use std::thread;
use std::time::Duration;

use self::termion::raw::IntoRawMode;
use self::termion::async_stdin;

pub fn write(data: &[u8]) -> String {
    let mut stdin = async_stdin().bytes();
    let mut stdout = stdout().lock().into_raw_mode().unwrap();
    stdout.write(data);
    let _ = stdout.flush();
    thread::sleep(Duration::from_millis(100));
    let mut buf = vec![];
    while let Some(Ok(v)) = stdin.next() {
        buf.push(v)
    }
    String::from_utf8(buf).map_or("".into(), |i| i)
}
