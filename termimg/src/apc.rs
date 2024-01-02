extern crate base64;
extern crate termion;
use self::termion::async_stdin;
use self::termion::raw::IntoRawMode;
use std::{
    collections::HashMap,
    io::{stdout, Read, Write},
    thread,
    time::Duration,
};

use self::base64::Engine;

pub enum ControlValue {
    Str(String),
    U16(u16),
}
pub struct APC {
    contro_data: HashMap<String, ControlValue>,
    payload: Vec<u8>,
}

impl APC {
    pub fn new() -> APC {
        APC {
            contro_data: HashMap::new(),
            payload: vec![],
        }
    }
    pub fn add_control_field(self: &mut Self, field: &str, value: ControlValue) -> &mut Self {
        self.contro_data.insert(field.into(), value);
        self
    }
    pub fn set_payload(self: &mut Self, data: &[u8]) -> &mut Self {
        let _ = self.payload.write(data);
        self
    }
    pub fn set_payload_base64(self: &mut Self, data: &[u8]) -> &mut Self {
        let eng = base64::engine::general_purpose::STANDARD;
        let encoded = eng.encode(data);
        let _ = self.payload.write(encoded.as_bytes());
        self
    }
    pub fn get(self: &Self) -> Vec<u8> {
        let mut data = vec![0x1b];
        let _ = data.write(b"_G");
        for (k, &ref v) in self.contro_data.iter() {
            let _ = data.write(k.as_bytes());
            data.push(b'=');
            match v {
                ControlValue::Str(v) => {
                    let _ = data.write(v.as_bytes());
                }
                ControlValue::U16(v) => {
                    let _ = data.write(v.to_string().as_bytes());
                }
            }
            data.push(b',');
        }
        let last_ind = data.len() - 1;
        if data[last_ind] == b',' {
            data[last_ind] = b';'
        } else {
            data.push(b';');
        }
        let _ = data.write(self.payload.as_slice());
        data.push(0x1b);
        data.push(b'\\');
        data
    }
    pub fn write(self: &Self) -> String {
        let mut stdin = async_stdin().bytes();
        let mut stdout = stdout().lock().into_raw_mode().unwrap();
        stdout.write(self.get().as_slice()).unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(100));
        let mut buf = vec![];
        while let Some(Ok(v)) = stdin.next() {
            buf.push(v)
        }
        String::from_utf8(buf).map_or("".into(), |i| i)
    }
}

pub fn write(apcs: &Vec<&APC>) -> String {
    let mut stdin = async_stdin().bytes();
    let mut stdout = stdout().lock().into_raw_mode().unwrap();
    for apc in apcs {
        let _ = stdout.write(apc.get().as_slice());
    }
    let _ = stdout.flush();
    thread::sleep(Duration::from_millis(100));
    let mut buf = vec![];
    while let Some(Ok(v)) = stdin.next() {
        buf.push(v)
    }
    String::from_utf8(buf).map_or("".into(), |i| i)
}
