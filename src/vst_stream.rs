use std::io::{ErrorKind, Read, Result, Write};
use vst3_com::{c_void, ComPtr};
use vst3_sys::base::{kIBSeekCur, kIBSeekEnd, kIBSeekSet, kResultOk, IBStream};

pub enum SeekMode {
    Set,
    RelativeCurrent,
    End,
}

pub enum StreamDir {
    In,
    Out,
}

fn seek(stream: &ComPtr<dyn IBStream>, pos: i64, mode: SeekMode) -> Result<i64> {
    let mut p: i64 = 0;

    if unsafe {
        stream.seek(
            pos,
            match mode {
                SeekMode::Set => kIBSeekSet,
                SeekMode::RelativeCurrent => kIBSeekCur,
                SeekMode::End => kIBSeekEnd,
            },
            &mut p as *mut i64,
        )
    } == kResultOk
    {
        Ok(p)
    }
    else {
        Err(ErrorKind::Other.into())
    }
}

fn tell(stream: &ComPtr<dyn IBStream>) -> Result<i64> {
    let mut pos: i64 = 0;

    if unsafe { stream.tell(&mut pos as *mut i64) } == kResultOk {
        Ok(pos)
    }
    else {
        Err(ErrorKind::Other.into())
    }
}

pub trait VstStream {
    fn seek(&self, pos: i64, mode: SeekMode) -> Result<i64>;
    fn tell(&self) -> Result<i64>;
}

pub struct VstInStream {
    stream: ComPtr<dyn IBStream>,
}

impl VstInStream {
    pub fn new(stream: ComPtr<dyn IBStream>) -> Self { Self { stream } }
}

impl VstStream for VstInStream {
    fn seek(&self, pos: i64, mode: SeekMode) -> Result<i64> { seek(&self.stream, pos, mode) }
    fn tell(&self) -> Result<i64> { tell(&self.stream) }
}

impl Read for VstInStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut num_bytes_read = 0;
        unsafe { self.stream.read(buf.as_mut_ptr() as *mut c_void, buf.len() as i32, &mut num_bytes_read) };
        Ok(num_bytes_read as usize)
    }
}

pub struct VstOutStream {
    stream: ComPtr<dyn IBStream>,
}

impl VstOutStream {
    pub fn new(stream: ComPtr<dyn IBStream>) -> Self { Self { stream } }
}

impl VstStream for VstOutStream {
    fn seek(&self, pos: i64, mode: SeekMode) -> Result<i64> { seek(&self.stream, pos, mode) }
    fn tell(&self) -> Result<i64> { tell(&self.stream) }
}

impl Write for VstOutStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut num_bytes_written = 0;
        unsafe { self.stream.write(buf.as_ptr() as *const c_void, buf.len() as i32, &mut num_bytes_written) };
        Ok(num_bytes_written as usize)
    }

    fn flush(&mut self) -> Result<()> { Ok(()) }
}
