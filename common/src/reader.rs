pub struct LEReader<'a> {
    data: &'a [u8],
    offset: usize
}

pub type Result<T> = std::result::Result<T, String>;

impl LEReader<'_> {
    pub fn new(data: &[u8]) -> LEReader {
        LEReader {
            data,
            offset: 0
        }
    }
    
    pub fn next_u8(&mut self) -> Result<u8> {
        match self.data.get(self.offset) {
            None => Err(err_msg()),
            Some(v) => {
                self.offset += 1;
                Ok(*v)
            }
        }
    }
    
    pub fn next_u16(&mut self) -> Result<u16> {
        if self.offset + 2 > self.data.len() {
            Err(err_msg())
        } else {
            let res = unsafe {
                read_u16_le(&self.data[self.offset..])
            };
            self.offset += 2;
            Ok(res)
        }
    }

    pub fn next_u32(&mut self) -> Result<u32> {
        if self.offset + 4 > self.data.len() {
            Err(err_msg())
        } else {
            let res = unsafe {
                read_u32_le(&self.data[self.offset..])
            };
            self.offset += 4;
            Ok(res)
        }
    }

    pub fn next_u64(&mut self) -> Result<u64> {
        if self.offset + 8 > self.data.len() {
            Err(err_msg())
        } else {
            let res = unsafe {
                read_u64_le(&self.data[self.offset..])
            };
            self.offset += 8;
            Ok(res)
        }
    }
    
    pub fn read_to(&mut self, to: &mut Vec<u8>, len: usize) -> Result<()> {
        if self.offset + len > self.data.len() {
            Err(err_msg())
        } else {
            to.extend_from_slice(&self.data[self.offset .. self.offset + len]);
            self.offset += len;
            Ok(())
        }
    }
    
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }
}

fn err_msg() -> String {
    "end of input".to_owned()
}

// unsafe fn read_u16_le(data: &[u8]) -> u16 {
//     u16::from_le(*(data.as_ptr() as *const u16))
// }
// 
// unsafe fn read_u32_le(data: &[u8]) -> u32 {
//     u32::from_le(*(data.as_ptr() as *const u32))
// }
// 
// unsafe fn read_u64_le(data: &[u8]) -> u64 {
//     u64::from_le(*(data.as_ptr() as *const u64))
// }

unsafe fn read_u16_le(data: &[u8]) -> u16 {
    let b0 = *data.get_unchecked(0) as u16;
    let b1 = *data.get_unchecked(1) as u16;
    b1 << 8 | b0
}

unsafe fn read_u32_le(data: &[u8]) -> u32 {
    let b0 = *data.get_unchecked(0) as u32;
    let b1 = *data.get_unchecked(1) as u32;
    let b2 = *data.get_unchecked(2) as u32;
    let b3 = *data.get_unchecked(3) as u32;
    b3 << 24 | b2 << 16 | b1 << 8 | b0
}

unsafe fn read_u64_le(data: &[u8]) -> u64 {
    let b0 = *data.get_unchecked(0) as u64;
    let b1 = *data.get_unchecked(1) as u64;
    let b2 = *data.get_unchecked(2) as u64;
    let b3 = *data.get_unchecked(3) as u64;
    let b4 = *data.get_unchecked(4) as u64;
    let b5 = *data.get_unchecked(5) as u64;
    let b6 = *data.get_unchecked(6) as u64;
    let b7 = *data.get_unchecked(7) as u64;
    b7 << 56 | b6 << 48 | b5 << 40 | b4 << 32 | b3 << 24 | b2 << 16 | b1 << 8 | b0
}