//! MessageSystem wire codec (LEB128 / Reference / SimpleFrame + Envelope).

pub struct WireReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

#[derive(Default)]
pub struct WireWriter {
    buf: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WireError {
    UnexpectedEof,
    VarintOverflow,
    InvalidUtf8,
}

pub type WireResult<T> = Result<T, WireError>;

impl<'a> WireReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    fn take(&mut self, n: usize) -> WireResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(WireError::UnexpectedEof);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn read_u8(&mut self) -> WireResult<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn read_bool(&mut self) -> WireResult<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> WireResult<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn read_i16(&mut self) -> WireResult<i16> {
        Ok(self.read_u16()? as i16)
    }

    pub fn read_u32(&mut self) -> WireResult<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_i32(&mut self) -> WireResult<i32> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_u64(&mut self) -> WireResult<u64> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn read_i64(&mut self) -> WireResult<i64> {
        Ok(self.read_u64()? as i64)
    }

    pub fn read_f32(&mut self) -> WireResult<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> WireResult<f64> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_var_u32(&mut self) -> WireResult<u32> {
        let mut result: u32 = 0;
        for i in 0..5 {
            let byte = self.read_u8()?;
            if i == 4 {
                if byte & 0xF0 != 0 {
                    return Err(WireError::VarintOverflow);
                }
                result |= (byte as u32) << 28;
                return Ok(result);
            }
            result |= ((byte & 0x7F) as u32) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(WireError::VarintOverflow)
    }

    pub fn read_var_u64(&mut self) -> WireResult<u64> {
        let mut result: u64 = 0;
        for i in 0..10 {
            let byte = self.read_u8()?;
            if i == 9 {
                result |= (byte as u64) << 63;
                return Ok(result);
            }
            result |= ((byte & 0x7F) as u64) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(WireError::VarintOverflow)
    }

    pub fn read_var_i32(&mut self) -> WireResult<i32> {
        Ok(zag32(self.read_var_u32()?))
    }

    pub fn read_var_i64(&mut self) -> WireResult<i64> {
        Ok(zag64(self.read_var_u64()?))
    }

    pub fn read_string(&mut self) -> WireResult<Option<String>> {
        if self.read_u8()? == 0 {
            return Ok(None);
        }
        let byte_count = self.read_var_i32()? as usize;
        let bytes = self.take(byte_count)?;
        core::str::from_utf8(bytes)
            .map(|s| Some(s.to_string()))
            .map_err(|_| WireError::InvalidUtf8)
    }

    pub fn read_bytes(&mut self, n: usize) -> WireResult<&'a [u8]> {
        self.take(n)
    }
}

impl WireWriter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_bool(&mut self, v: bool) {
        self.buf.push(v as u8);
    }

    pub fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i16(&mut self, v: i16) {
        self.write_u16(v as u16);
    }

    pub fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.write_u32(v as u32);
    }

    /// `Rts.Serialization` `WriteEnum` for a default (Int32) `[Flags]` enum:
    /// `TypeCode.Int32` (9) + little-endian i32. A raw u32 is read as TypeCode=first
    /// byte and falls through to `default(None)` — unused bits such as Tech Tree 0x20
    /// never reach Prism.
    pub fn write_rts_enum_i32(&mut self, value: i32) {
        self.write_u8(9);
        self.write_i32(value);
    }

    pub fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i64(&mut self, v: i64) {
        self.write_u64(v as u64);
    }

    pub fn write_f32(&mut self, v: f32) {
        self.write_u32(v.to_bits());
    }

    pub fn write_f64(&mut self, v: f64) {
        self.write_u64(v.to_bits());
    }

    pub fn write_var_u32(&mut self, mut v: u32) {
        loop {
            let mut byte = (v as u8) & 0x7F;
            v >>= 7;
            if v != 0 {
                byte |= 0x80;
            }
            self.buf.push(byte);
            if v == 0 {
                break;
            }
        }
    }

    pub fn write_var_u64(&mut self, mut v: u64) {
        loop {
            let mut byte = (v as u8) & 0x7F;
            v >>= 7;
            if v != 0 {
                byte |= 0x80;
            }
            self.buf.push(byte);
            if v == 0 {
                break;
            }
        }
    }

    pub fn write_var_i32(&mut self, v: i32) {
        self.write_var_u32(zig32(v));
    }

    pub fn write_var_i64(&mut self, v: i64) {
        self.write_var_u64(zig64(v));
    }

    pub fn write_string(&mut self, v: Option<&str>) {
        match v {
            None => self.buf.push(0),
            Some(s) => {
                self.buf.push(1);
                let bytes = s.as_bytes();
                self.write_var_i32(bytes.len() as i32);
                self.buf.extend_from_slice(bytes);
            }
        }
    }

    pub fn write_bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    pub fn write_ref_array_u32(&mut self, values: &[u32]) {
        self.write_u8(1);
        self.write_var_i32(values.len() as i32);
        for v in values {
            self.write_u32(*v);
        }
    }

    pub fn write_ref_array_u64(&mut self, values: &[u64]) {
        self.write_u8(1);
        self.write_var_i32(values.len() as i32);
        for v in values {
            self.write_u64(*v);
        }
    }

    pub fn write_ref_array_f32(&mut self, values: &[f32]) {
        self.write_u8(1);
        self.write_var_i32(values.len() as i32);
        for v in values {
            self.write_f32(*v);
        }
    }

    pub fn write_ref_array_len(&mut self, len: usize) {
        self.write_u8(1);
        self.write_var_i32(len as i32);
    }
}

fn zig32(v: i32) -> u32 {
    ((v << 1) ^ (v >> 31)) as u32
}

fn zig64(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

fn zag32(v: u32) -> i32 {
    ((v >> 1) as i32) ^ -((v & 1) as i32)
}

fn zag64(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    pub sender: Vec<u8>,
    pub receiver: Vec<u8>,
    pub type_id: u16,
    pub payload: Vec<u8>,
}

impl Envelope {
    pub fn try_read(buf: &[u8]) -> WireResult<Option<(Envelope, usize)>> {
        if buf.len() < 4 {
            return Ok(None);
        }
        let envelope_size = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if buf.len() - 4 < envelope_size {
            return Ok(None);
        }
        let mut r = WireReader::new(&buf[4..4 + envelope_size]);
        let sender_len = r.read_u16()? as usize;
        let sender = r.read_bytes(sender_len)?.to_vec();
        let recv_len = r.read_u16()? as usize;
        let receiver = r.read_bytes(recv_len)?.to_vec();
        let type_id = r.read_u16()?;
        let payload_len = r.read_u32()? as usize;
        let payload = r.read_bytes(payload_len)?.to_vec();
        Ok(Some((
            Envelope {
                sender,
                receiver,
                type_id,
                payload,
            },
            4 + envelope_size,
        )))
    }

    pub fn write(&self) -> Vec<u8> {
        let mut body = WireWriter::new();
        body.write_u16(self.sender.len() as u16);
        body.write_bytes(&self.sender);
        body.write_u16(self.receiver.len() as u16);
        body.write_bytes(&self.receiver);
        body.write_u16(self.type_id);
        body.write_u32(self.payload.len() as u32);
        body.write_bytes(&self.payload);

        let body = body.into_bytes();
        let mut out = WireWriter::new();
        out.write_u32(body.len() as u32);
        out.write_bytes(&body);
        out.into_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleFrame {
    pub type_id: u16,
    pub payload: Vec<u8>,
}

impl SimpleFrame {
    pub fn try_read(buf: &[u8]) -> WireResult<Option<(SimpleFrame, usize)>> {
        if buf.len() < 6 {
            return Ok(None);
        }
        let type_id = u16::from_le_bytes([buf[0], buf[1]]);
        let payload_len = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]) as usize;
        if buf.len() - 6 < payload_len {
            return Ok(None);
        }
        Ok(Some((
            SimpleFrame {
                type_id,
                payload: buf[6..6 + payload_len].to_vec(),
            },
            6 + payload_len,
        )))
    }

    pub fn write(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(6 + self.payload.len());
        out.extend_from_slice(&self.type_id.to_le_bytes());
        out.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    pub fn from_wire(bytes: &[u8]) -> Self {
        let (frame, _) = Self::try_read(bytes)
            .expect("valid wire bytes")
            .expect("complete frame");
        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_frame_roundtrip() {
        let f = SimpleFrame {
            type_id: 1,
            payload: vec![0xAA, 0xBB, 0xCC],
        };
        let bytes = f.write();
        assert_eq!(&bytes[0..2], &1u16.to_le_bytes());
        assert_eq!(&bytes[2..6], &3u32.to_le_bytes());
        let (decoded, consumed) = SimpleFrame::try_read(&bytes).unwrap().unwrap();
        assert_eq!(decoded, f);
        assert_eq!(consumed, bytes.len());
        // Partial header / partial payload -> None.
        assert_eq!(SimpleFrame::try_read(&bytes[..5]).unwrap(), None);
        assert_eq!(SimpleFrame::try_read(&bytes[..bytes.len() - 1]).unwrap(), None);
    }

    #[test]
    fn fixed_ints_roundtrip_little_endian() {
        let mut w = WireWriter::new();
        w.write_u16(0x1234);
        w.write_u32(0xDEADBEEF);
        w.write_i64(-2);
        w.write_f32(1.5);
        // Confirm little-endian byte order for the u16.
        assert_eq!(&w.as_slice()[0..2], &[0x34, 0x12]);
        let mut r = WireReader::new(w.as_slice());
        assert_eq!(r.read_u16().unwrap(), 0x1234);
        assert_eq!(r.read_u32().unwrap(), 0xDEADBEEF);
        assert_eq!(r.read_i64().unwrap(), -2);
        assert_eq!(r.read_f32().unwrap(), 1.5);
    }

    #[test]
    fn varint_unsigned_matches_leb128() {
        let mut w = WireWriter::new();
        w.write_var_u32(0);
        w.write_var_u32(127);
        w.write_var_u32(128);
        w.write_var_u32(300);
        w.write_var_u32(u32::MAX);
        // 0 -> [0x00]; 127 -> [0x7F]; 128 -> [0x80,0x01]; 300 -> [0xAC,0x02].
        assert_eq!(&w.as_slice()[0..1], &[0x00]);
        let mut r = WireReader::new(w.as_slice());
        assert_eq!(r.read_var_u32().unwrap(), 0);
        assert_eq!(r.read_var_u32().unwrap(), 127);
        assert_eq!(r.read_var_u32().unwrap(), 128);
        assert_eq!(r.read_var_u32().unwrap(), 300);
        assert_eq!(r.read_var_u32().unwrap(), u32::MAX);
    }

    #[test]
    fn varint_signed_zigzag_roundtrip() {
        for v in [0i32, -1, 1, -2, 2, i32::MIN, i32::MAX, 123456, -123456] {
            let mut w = WireWriter::new();
            w.write_var_i32(v);
            let mut r = WireReader::new(w.as_slice());
            assert_eq!(r.read_var_i32().unwrap(), v, "i32 zigzag roundtrip {v}");
        }
        // zigzag encoding: -1 -> 1, 1 -> 2, -2 -> 3 (byte 0 low bits).
        let mut w = WireWriter::new();
        w.write_var_i32(-1);
        assert_eq!(w.as_slice()[0], 1);
        let mut w = WireWriter::new();
        w.write_var_i32(1);
        assert_eq!(w.as_slice()[0], 2);
    }

    #[test]
    fn var_i64_roundtrip() {
        for v in [0i64, -1, 1, i64::MIN, i64::MAX, 20003656600213] {
            let mut w = WireWriter::new();
            w.write_var_i64(v);
            let mut r = WireReader::new(w.as_slice());
            assert_eq!(r.read_var_i64().unwrap(), v);
        }
    }

    #[test]
    fn string_reference_and_null() {
        let mut w = WireWriter::new();
        w.write_string(Some("RtsBlazeClient"));
        w.write_string(None);
        w.write_string(Some(""));
        // present marker 1, then zigzag-varint(14)=28, then bytes.
        assert_eq!(w.as_slice()[0], 1);
        assert_eq!(w.as_slice()[1], 28);
        let mut r = WireReader::new(w.as_slice());
        assert_eq!(r.read_string().unwrap().as_deref(), Some("RtsBlazeClient"));
        assert_eq!(r.read_string().unwrap(), None);
        assert_eq!(r.read_string().unwrap().as_deref(), Some(""));
    }

    #[test]
    fn envelope_roundtrip() {
        let env = Envelope {
            sender: vec![],
            receiver: vec![],
            type_id: 7,
            payload: vec![1, 2, 3, 4, 5],
        };
        let bytes = env.write();
        // envelopeSize = 2(senderLen)+0 + 2(recvLen)+0 + 2(typeId) + 4(payloadLen) + 5(payload) = 15.
        assert_eq!(&bytes[0..4], &15u32.to_le_bytes());
        let (decoded, consumed) = Envelope::try_read(&bytes).unwrap().unwrap();
        assert_eq!(decoded, env);
        assert_eq!(consumed, bytes.len());
    }

    #[test]
    fn envelope_partial_returns_none() {
        let env = Envelope {
            sender: vec![9, 9],
            receiver: vec![],
            type_id: 1,
            payload: vec![0; 10],
        };
        let bytes = env.write();
        // Feed everything except the last byte -> needs more.
        assert_eq!(Envelope::try_read(&bytes[..bytes.len() - 1]).unwrap(), None);
        // Fewer than 4 bytes (can't even read the size) -> needs more.
        assert_eq!(Envelope::try_read(&bytes[..3]).unwrap(), None);
        // Full frame decodes with sender bytes preserved.
        let (decoded, _) = Envelope::try_read(&bytes).unwrap().unwrap();
        assert_eq!(decoded.sender, vec![9, 9]);
    }
}
