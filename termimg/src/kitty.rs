use std::{
    fs::File,
    io::{stdout, Read, Stdin, Write},
    os::fd::FromRawFd,
    path::Path,
    thread,
    time::Duration,
};
extern crate atty;
extern crate base64;
extern crate infer;
extern crate nix;
extern crate termion;
use self::base64::Engine;
use self::nix::libc::O_RDWR;
use self::nix::libc::{O_CREAT, S_IROTH, S_IWUSR};
use self::nix::libc::{S_IRUSR, S_IXUSR};
use self::termion::raw::IntoRawMode;
use image::{DynamicImage, EncodableLayout, GenericImageView};
use self::termion::cursor::DetectCursorPos;

use crate::{
    apc::{ControlValue, APC},
    graphic::{Graphic, TerminalSize},
    utils::{get_image, has_alpha, prepare_img},
};

pub struct Kitty;
impl Graphic for Kitty {
    fn display(&self, img: &DynamicImage) -> Result<(), String> {
        let terminal_size = self.size();
        let fns: [(
            bool,
            fn(&DynamicImage, Option<TerminalSize>) -> Result<(), String>,
        ); 2] = [
            (is_shared_mem_supported(), show_by_shared_memory),
            (is_direct_supported(), show_by_direct_data),
        ];
        for &(supported, f) in &fns {
            if supported {
                let r = f(img, terminal_size.clone());
                if r.is_ok() {
                    return r;
                }
            }
        }
        Err("Unsupported".to_owned())
    }

    fn supported(&self) -> bool {
        is_direct_supported() || is_shared_mem_supported()
    }

    fn name(&self) -> &'static str {
        "kitty"
    }
}

fn set_showing_position(size: TerminalSize, img_width: u32) -> () {
    let TerminalSize {
        width,
        height,
        rows,
        cols,
    } = size;
    let cell_width = (width as f32) / (cols as f32);
    let left = ((width - img_width as u16) as f32) / 2.0;
    let cell_index = (left / cell_width) as u16;
    horizental_move_cur(cell_index);
}
fn show_by_direct_data(img: &DynamicImage, size: Option<TerminalSize>) -> Result<(), String> {
    let (w, h) = img.dimensions();
    if size.is_some() {
        set_showing_position(size.unwrap(), w);
    }
    let trans = Transimisson::new()
        .data_format(if has_alpha(&img) {
            DataFormat::RGBA
        } else {
            DataFormat::RGB
        })
        .action(Action::ImmediatelyShow)
        .transmission_type(TransmissionType::Direct(
            img.to_bytes().as_bytes(),
            (w as u16, h as u16),
        ));
    trans.transfer();
    Ok(())
}

fn show_by_shared_memory(img: &DynamicImage, size: Option<TerminalSize>) -> Result<(), String> {
    let (w, h) = img.dimensions();
    if size.is_some() {
        set_showing_position(size.unwrap(), w);
    }
    let trans = Transimisson::new()
        .data_format(if has_alpha(&img) {
            DataFormat::RGBA
        } else {
            DataFormat::RGB
        })
        .action(Action::ImmediatelyShow)
        .transmission_type(TransmissionType::SharedMemory(
            "__termimg_image_object__".to_owned(),
            (w as u16, h as u16),
        ));
    let _ = nix::sys::mman::shm_unlink("__termimg_image_object__");
    let id = nix::sys::mman::shm_open(
        "__termimg_image_object__",
        unsafe { nix::fcntl::OFlag::from_bits_unchecked(O_CREAT | O_RDWR) },
        nix::sys::stat::Mode::from_bits_truncate(S_IRUSR | S_IWUSR | S_IXUSR),
    );
    if id.is_err() {
        eprintln!("{:?}", id);
        Err("Failed to create mem".to_owned())
    } else {
        let mut file = unsafe { File::from_raw_fd(id.unwrap()) };
        file.write(img.to_bytes().as_bytes())
            .map_err(|err| err.to_string())
            .map(|_| {
                let _ = trans.transfer();
            })
    }
}

fn horizental_move_cur(u: u16) {
    let mut stdout = stdout().lock().into_raw_mode().unwrap();
    let pos = stdout.cursor_pos();
    let _ = if pos.is_ok() {
        let (_,y) = pos.unwrap();
        write!(stdout,"{}",self::termion::cursor::Goto(u,y))
    } else {
        println!("");
        write!(stdout, "{}", self::termion::cursor::Right(u))
    };
}

enum TransmissionType<'a> {
    Direct(&'a [u8], (u16, u16)),
    RegularFile(String),
    Temp(String),
    SharedMemory(String, (u16, u16)),
}
enum DataFormat {
    Png,
    RGB,
    RGBA,
}
enum Action {
    Query,
    ImmediatelyShow,
    Placement,
}

fn get_data_format(fmt: DataFormat) -> ControlValue {
    ControlValue::U16(match fmt {
        DataFormat::Png => 100u16,
        DataFormat::RGB => 24u16,
        DataFormat::RGBA => 32u16,
    })
}

fn is_regular_file_supported() -> bool {
    let trans = Transimisson::new();
    let resp = trans
        .transmission_type(TransmissionType::RegularFile("<path>".to_owned()))
        .action(Action::Query)
        .data_format(DataFormat::Png)
        .transfer();
    resp.len() > 0 && !resp.contains("ENOTSUPPORTED")
}
fn is_direct_supported() -> bool {
    let trans = Transimisson::new();
    let resp = trans
        .transmission_type(TransmissionType::Direct(&[255, 255, 255], (1, 1)))
        .action(Action::Query)
        .data_format(DataFormat::RGB)
        .transfer();
    resp.len() > 0 && !resp.contains("ENOTSUPPORTED")
}
fn is_shared_mem_supported() -> bool {
    let trans = Transimisson::new();
    let resp = trans
        .transmission_type(TransmissionType::SharedMemory("__".to_owned(), (1, 1)))
        .action(Action::Query)
        .data_format(DataFormat::RGB)
        .transfer();

    resp.len() > 0 && !resp.contains("ENOTSUPPORTED")
}

struct Transimisson {
    apc: APC,
    chunks: Vec<APC>,
    id: u16,
}
impl Transimisson {
    fn new() -> Transimisson {
        Transimisson {
            apc: APC::new(),
            chunks: vec![],
            id: 1,
        }
    }
    fn transmission_type(mut self: Self, t: TransmissionType) -> Self {
        match t {
            TransmissionType::Temp(data) => {
                self.apc
                    .add_control_field("t", ControlValue::Str("t".into()))
                    .set_payload_base64(data.as_bytes());
                ()
            }
            TransmissionType::SharedMemory(name, (w, h)) => {
                self.apc
                    .add_control_field("s", ControlValue::U16(w))
                    .add_control_field("v", ControlValue::U16(h))
                    .add_control_field("t", ControlValue::Str("s".to_owned()));
                self.apc.set_payload_base64(name.as_bytes());
            }
            TransmissionType::Direct(data, (w, h)) => {
                let eng = base64::engine::general_purpose::STANDARD;
                let encoded = eng.encode(data);
                let chunk_size = 4096;
                self.apc
                    .add_control_field("t", ControlValue::Str("d".into()))
                    .add_control_field("s", ControlValue::U16(w))
                    .add_control_field("v", ControlValue::U16(h));
                if encoded.len() < chunk_size {
                    self.apc.set_payload(encoded.as_bytes());
                } else {
                    self.apc.add_control_field("m", ControlValue::U16(1));
                    self.chunks = encoded
                        .as_bytes()
                        .chunks(chunk_size)
                        .map(|chunk| {
                            let mut apc = APC::new();
                            apc.add_control_field("m", ControlValue::U16(1));
                            apc.set_payload(chunk);
                            apc
                        })
                        .collect();
                    let last_ind = self.chunks.len() - 1;
                    self.chunks[last_ind].add_control_field("m", ControlValue::U16(0));
                }
            }
            TransmissionType::RegularFile(file) => {
                self.apc
                    .add_control_field("t", ControlValue::Str("f".into()))
                    .set_payload_base64(file.as_bytes());
            }
        };
        self
    }
    fn data_format(mut self: Self, fmt: DataFormat) -> Self {
        self.apc.add_control_field("f", get_data_format(fmt));
        self
    }
    fn action(mut self: Self, action: Action) -> Self {
        self.apc.add_control_field(
            "a",
            match action {
                Action::Query => ControlValue::Str("q".to_owned()),
                Action::ImmediatelyShow => ControlValue::Str("T".to_owned()),
                Action::Placement => ControlValue::Str("p".to_owned()),
            },
        );
        self
    }
    fn row(mut self: Self, row: u16) -> Self {
        self.apc.add_control_field("r", ControlValue::U16(row));
        self
    }
    fn col(mut self: Self, col: u16) -> Self {
        self.apc.add_control_field("c", ControlValue::U16(col));
        self
    }
    fn transfer(mut self: Self) -> String {
        self.apc.add_control_field("i", ControlValue::U16(self.id));
        if self.chunks.len() > 0 {
            let mut apcs = vec![&self.apc];
            for ch in self.chunks.iter() {
                apcs.push(ch)
            }
            crate::apc::write(&apcs);
            "".into()
        } else {
            self.apc.write()
        }
    }
}
