use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreSource {
    Chrome,
    Edge,
}

/// Validates a 32-character lowercase Chrome/Edge extension ID (`'a'..='p'`).
pub fn is_valid_chrome_ext_id(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 32 && bytes.iter().all(|b| matches!(b, b'a'..=b'p'))
}

/// Parses either a raw 32-character extension ID or a Chrome Web Store / Edge Add-ons URL.
pub fn parse_store_input(raw: &str) -> Option<(String, StoreSource)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if is_valid_chrome_ext_id(trimmed) {
        return Some((trimmed.to_string(), StoreSource::Chrome));
    }
    let clean = trimmed.split(['?', '#']).next().unwrap_or("");
    let no_scheme = clean
        .strip_prefix("https://")
        .or_else(|| clean.strip_prefix("http://"))
        .unwrap_or(clean);
    let mut parts = no_scheme.splitn(2, '/');
    let host = parts.next().unwrap_or("").to_ascii_lowercase();
    let path = parts.next().unwrap_or("");
    let source = if host.contains("microsoftedge.microsoft.com") {
        StoreSource::Edge
    } else {
        StoreSource::Chrome
    };
    for seg in path.split('/').rev() {
        let candidate = seg.trim();
        if is_valid_chrome_ext_id(candidate) {
            return Some((candidate.to_string(), source));
        }
    }
    None
}

/// Pure-Rust FIPS 180-4 SHA-256 hash.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut padded = Vec::with_capacity(data.len() + 72);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while (padded.len() + 8) % 64 != 0 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (dst, add) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *dst = dst.wrapping_add(add);
        }
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Converts a 16-byte CRX ID digest prefix into a 32-character lowercase `'a'..='p'` Chrome extension ID.
pub fn ext_id_from_crx_id_bytes(bytes16: &[u8]) -> Option<String> {
    if bytes16.len() != 16 {
        return None;
    }
    let mut id = String::with_capacity(32);
    for &b in bytes16 {
        id.push((b'a' + (b >> 4)) as char);
        id.push((b'a' + (b & 0x0f)) as char);
    }
    Some(id)
}

/// Derives Chrome's canonical 32-char extension ID (`SHA-256(pubkey)[0..16]` in `'a'..='p'` nibbles).
pub fn ext_id_from_public_key(pubkey_der: &[u8]) -> String {
    let digest = sha256(pubkey_der);
    ext_id_from_crx_id_bytes(&digest[..16]).unwrap_or_default()
}

fn read_varint(buf: &[u8], pos: &mut usize) -> Result<u64, String> {
    let mut val = 0u64;
    let mut shift = 0u32;
    for _ in 0..10 {
        if *pos >= buf.len() {
            return Err("Truncated CRX3 protobuf varint".to_string());
        }
        let b = buf[*pos];
        *pos += 1;
        val |= ((b & 0x7f) as u64) << shift;
        if (b & 0x80) == 0 {
            return Ok(val);
        }
        shift += 7;
    }
    Err("Malformed CRX3 protobuf varint".to_string())
}

fn skip_proto_field(buf: &[u8], pos: &mut usize, wire_type: u64) -> Result<(), String> {
    match wire_type {
        0 => {
            let _ = read_varint(buf, pos)?;
            Ok(())
        }
        1 => {
            let end = pos
                .checked_add(8)
                .filter(|&p| p <= buf.len())
                .ok_or_else(|| "Truncated CRX3 protobuf 64-bit field".to_string())?;
            *pos = end;
            Ok(())
        }
        2 => {
            let len = usize::try_from(read_varint(buf, pos)?)
                .map_err(|_| "CRX3 protobuf field length overflow".to_string())?;
            let end = pos
                .checked_add(len)
                .filter(|&p| p <= buf.len())
                .ok_or_else(|| "Truncated CRX3 protobuf bytes field".to_string())?;
            *pos = end;
            Ok(())
        }
        5 => {
            let end = pos
                .checked_add(4)
                .filter(|&p| p <= buf.len())
                .ok_or_else(|| "Truncated CRX3 protobuf 32-bit field".to_string())?;
            *pos = end;
            Ok(())
        }
        _ => Err(format!("Unsupported CRX3 protobuf wire type {}", wire_type)),
    }
}

fn read_proto_bytes<'a>(buf: &'a [u8], pos: &mut usize) -> Result<&'a [u8], String> {
    let len = usize::try_from(read_varint(buf, pos)?)
        .map_err(|_| "CRX3 protobuf length overflow".to_string())?;
    let end = pos
        .checked_add(len)
        .filter(|&p| p <= buf.len())
        .ok_or_else(|| "Truncated CRX3 protobuf length-delimited field".to_string())?;
    let slice = &buf[*pos..end];
    *pos = end;
    Ok(slice)
}

fn parse_asymmetric_key_proof(buf: &[u8]) -> Result<Option<String>, String> {
    let mut pos = 0usize;
    let mut pubkey: Option<&[u8]> = None;
    let mut sig: Option<&[u8]> = None;
    while pos < buf.len() {
        let tag = read_varint(buf, &mut pos)?;
        let field = tag >> 3;
        let wire = tag & 0x7;
        match (field, wire) {
            (1, 2) => pubkey = Some(read_proto_bytes(buf, &mut pos)?),
            (2, 2) => sig = Some(read_proto_bytes(buf, &mut pos)?),
            _ => skip_proto_field(buf, &mut pos, wire)?,
        }
    }
    if let (Some(pk), Some(s)) = (pubkey, sig) {
        if !pk.is_empty() && !s.is_empty() {
            return Ok(Some(ext_id_from_public_key(pk)));
        }
    }
    Ok(None)
}

fn parse_signed_data_crx_id(buf: &[u8]) -> Result<Option<String>, String> {
    let mut pos = 0usize;
    let mut signed_id: Option<String> = None;
    while pos < buf.len() {
        let tag = read_varint(buf, &mut pos)?;
        let field = tag >> 3;
        let wire = tag & 0x7;
        if field == 1 && wire == 2 {
            let bytes = read_proto_bytes(buf, &mut pos)?;
            signed_id = ext_id_from_crx_id_bytes(bytes);
        } else {
            skip_proto_field(buf, &mut pos, wire)?;
        }
    }
    Ok(signed_id)
}

fn parse_crx3_header_ext_ids(hdr: &[u8]) -> Result<(Vec<String>, Option<String>), String> {
    let mut pos = 0usize;
    let mut proof_ext_ids = Vec::new();
    let mut signed_crx_id = None;
    while pos < hdr.len() {
        let tag = read_varint(buf_hdr(hdr), &mut pos)?;
        let field = tag >> 3;
        let wire = tag & 0x7;
        match (field, wire) {
            (2, 2) | (3, 2) => {
                let proof = read_proto_bytes(hdr, &mut pos)?;
                if let Some(id) = parse_asymmetric_key_proof(proof)? {
                    proof_ext_ids.push(id);
                }
            }
            (10000, 2) => {
                let signed = read_proto_bytes(hdr, &mut pos)?;
                if let Some(id) = parse_signed_data_crx_id(signed)? {
                    signed_crx_id = Some(id);
                }
            }
            _ => skip_proto_field(hdr, &mut pos, wire)?,
        }
    }
    Ok((proof_ext_ids, signed_crx_id))
}

#[inline]
fn buf_hdr(hdr: &[u8]) -> &[u8] {
    hdr
}

enum CrxContainer<'a> {
    Zip,
    Crx2 { pk: &'a [u8], sig: &'a [u8] },
    Crx3 { hdr: &'a [u8] },
}

fn parse_crx_container(raw: &[u8]) -> Result<(&[u8], CrxContainer<'_>), String> {
    if raw.len() < 4 {
        return Err("Archive file is empty or truncated".to_string());
    }
    if raw.starts_with(b"PK\x03\x04") || raw.starts_with(b"PK\x05\x06") {
        return Ok((raw, CrxContainer::Zip));
    }
    if raw.len() < 12 || &raw[..4] != b"Cr24" {
        return Err("Not a valid CRX or ZIP archive".to_string());
    }
    let version = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
    let (zip_off, container) = match version {
        3 => {
            let hdr_len = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
            let zip_off = 12usize
                .checked_add(hdr_len)
                .ok_or_else(|| "CRX3 header length overflow".to_string())?;
            if zip_off >= raw.len() {
                return Err("CRX header points past end of file".to_string());
            }
            (zip_off, CrxContainer::Crx3 { hdr: &raw[12..zip_off] })
        }
        2 => {
            if raw.len() < 16 {
                return Err("Truncated CRX2 header".to_string());
            }
            let pk_len = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
            let sig_len = u32::from_le_bytes([raw[12], raw[13], raw[14], raw[15]]) as usize;
            let pk_end = 16usize
                .checked_add(pk_len)
                .ok_or_else(|| "CRX2 header length overflow".to_string())?;
            let zip_off = pk_end
                .checked_add(sig_len)
                .ok_or_else(|| "CRX2 header length overflow".to_string())?;
            if zip_off >= raw.len() {
                return Err("CRX header points past end of file".to_string());
            }
            (
                zip_off,
                CrxContainer::Crx2 {
                    pk: &raw[16..pk_end],
                    sig: &raw[pk_end..zip_off],
                },
            )
        }
        other => {
            return Err(format!("Unsupported CRX format version: {}", other));
        }
    };
    let zip_slice = &raw[zip_off..];
    if !zip_slice.starts_with(b"PK\x03\x04") && !zip_slice.starts_with(b"PK\x05\x06") {
        return Err("CRX payload does not start with a valid ZIP signature".to_string());
    }
    Ok((zip_slice, container))
}

/// Strips a CRX2 or CRX3 header prefix and returns the underlying ZIP archive slice.
pub fn strip_crx_header(raw: &[u8]) -> Result<&[u8], String> {
    parse_crx_container(raw).map(|(zip, _)| zip)
}

/// Validates CRX2/CRX3 header proofs and checks public-key extension ID consistency, then returns the ZIP slice and detected extension ID.
pub fn verify_and_strip_crx<'a>(
    raw: &'a [u8],
    expected_ext_id: Option<&str>,
    require_crx_proof: bool,
) -> Result<(&'a [u8], Option<String>), String> {
    if let Some(exp) = expected_ext_id {
        if !is_valid_chrome_ext_id(exp) {
            return Err("Expected Chrome extension ID is invalid".to_string());
        }
    }
    let (zip_slice, container) = parse_crx_container(raw)?;
    match container {
        CrxContainer::Zip => {
            if require_crx_proof {
                return Err("Store package must be a signed CRX archive, not an unsigned ZIP".to_string());
            }
            Ok((zip_slice, expected_ext_id.map(|s| s.to_string())))
        }
        CrxContainer::Crx2 { pk, sig } => {
            if require_crx_proof && (pk.is_empty() || sig.is_empty()) {
                return Err("CRX2 header is missing public key or signature".to_string());
            }
            let derived_id = if !pk.is_empty() {
                Some(ext_id_from_public_key(pk))
            } else {
                None
            };
            if let (Some(exp), Some(ref got)) = (expected_ext_id, &derived_id) {
                if exp != got {
                    return Err(format!(
                        "CRX2 public key ID mismatch: expected {}, got {}",
                        exp, got
                    ));
                }
            }
            Ok((zip_slice, expected_ext_id.map(|s| s.to_string()).or(derived_id)))
        }
        CrxContainer::Crx3 { hdr } => {
            let (proof_ids, signed_crx_id) = parse_crx3_header_ext_ids(hdr)?;
            if require_crx_proof && proof_ids.is_empty() {
                return Err("CRX3 archive contains no valid AsymmetricKeyProof".to_string());
            }
            if let (Some(ref signed_id), true) = (&signed_crx_id, !proof_ids.is_empty()) {
                if !proof_ids.iter().any(|id| id == signed_id) {
                    return Err(
                        "CRX3 signed_header_data crx_id does not match any public key proof"
                            .to_string(),
                    );
                }
            }
            if let Some(exp) = expected_ext_id {
                if !proof_ids.is_empty() && !proof_ids.iter().any(|id| id == exp) {
                    return Err(format!(
                        "CRX3 public key ID mismatch: expected {}",
                        exp
                    ));
                }
                if let Some(ref signed_id) = signed_crx_id {
                    if signed_id != exp {
                        return Err(format!(
                            "CRX3 signed crx_id mismatch: expected {}, got {}",
                            exp, signed_id
                        ));
                    }
                }
            }
            let detected_id = expected_ext_id
                .map(|s| s.to_string())
                .or(signed_crx_id)
                .or_else(|| proof_ids.into_iter().next());
            Ok((zip_slice, detected_id))
        }
    }
}

struct BitReader<'a> {
    src: &'a [u8],
    pos: usize,
    bit_buf: u64,
    bit_cnt: u8,
}

impl<'a> BitReader<'a> {
    fn new(src: &'a [u8]) -> Self {
        Self {
            src,
            pos: 0,
            bit_buf: 0,
            bit_cnt: 0,
        }
    }

    #[inline]
    fn fill(&mut self, need: u8) -> Result<(), String> {
        while self.bit_cnt < need {
            if self.pos >= self.src.len() {
                return Err("Unexpected end of compressed stream".to_string());
            }
            self.bit_buf |= (self.src[self.pos] as u64) << self.bit_cnt;
            self.pos += 1;
            self.bit_cnt += 8;
        }
        Ok(())
    }

    #[inline]
    fn read_bit(&mut self) -> Result<u32, String> {
        if self.bit_cnt == 0 {
            self.fill(1)?;
        }
        let b = (self.bit_buf & 1) as u32;
        self.bit_buf >>= 1;
        self.bit_cnt -= 1;
        Ok(b)
    }

    #[inline]
    fn read_bits(&mut self, n: u8) -> Result<u32, String> {
        if n == 0 {
            return Ok(0);
        }
        if self.bit_cnt < n {
            self.fill(n)?;
        }
        let val = (self.bit_buf & ((1u64 << n) - 1)) as u32;
        self.bit_buf >>= n;
        self.bit_cnt -= n;
        Ok(val)
    }

    /// Rewinds buffered whole bytes and resets bit buffer for byte-aligned stored blocks.
    fn align_byte_slice(&mut self) {
        let unread_whole_bytes = (self.bit_cnt / 8) as usize;
        self.pos = self.pos.saturating_sub(unread_whole_bytes);
        self.bit_buf = 0;
        self.bit_cnt = 0;
    }

    fn read_aligned_bytes(&mut self, count: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(count)
            .ok_or_else(|| "DEFLATE stored length overflow".to_string())?;
        if end > self.src.len() {
            return Err("Truncated stored DEFLATE block".to_string());
        }
        let slice = &self.src[self.pos..end];
        self.pos = end;
        Ok(slice)
    }
}

const MAX_BITS: usize = 15;

struct HuffmanTable {
    counts: [u16; MAX_BITS + 1],
    symbols: Vec<u16>,
}

impl HuffmanTable {
    fn from_lengths(lengths: &[u8]) -> Result<Self, String> {
        let mut counts = [0u16; MAX_BITS + 1];
        for &len in lengths {
            let l = len as usize;
            if l > MAX_BITS {
                return Err("Invalid Huffman code length".to_string());
            }
            counts[l] += 1;
        }
        if counts[0] as usize == lengths.len() {
            return Ok(Self {
                counts,
                symbols: Vec::new(),
            });
        }
        let mut left: i32 = 1;
        for &c in &counts[1..=MAX_BITS] {
            left <<= 1;
            left -= c as i32;
            if left < 0 {
                return Err("Over-subscribed Huffman tree".to_string());
            }
        }
        let mut offs = [0u16; MAX_BITS + 1];
        for len in 1..MAX_BITS {
            offs[len + 1] = offs[len] + counts[len];
        }
        let total_assigned: usize = counts[1..].iter().map(|&c| c as usize).sum();
        let mut symbols = vec![0u16; total_assigned];
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                let l = len as usize;
                let idx = offs[l] as usize;
                offs[l] += 1;
                symbols[idx] = sym as u16;
            }
        }
        Ok(Self { counts, symbols })
    }

    #[inline]
    fn decode(&self, br: &mut BitReader<'_>) -> Result<u16, String> {
        let mut code: i32 = 0;
        let mut first: i32 = 0;
        let mut index: i32 = 0;
        for len in 1..=MAX_BITS {
            code |= br.read_bit()? as i32;
            let count = self.counts[len] as i32;
            if code - count < first {
                return self
                    .symbols
                    .get((index + (code - first)) as usize)
                    .copied()
                    .ok_or_else(|| "Huffman symbol out of range".to_string());
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("Invalid Huffman code in DEFLATE stream".to_string())
    }
}

fn fixed_tables() -> Result<(HuffmanTable, HuffmanTable), String> {
    let mut lit_lens = [0u8; 288];
    lit_lens[..144].fill(8);
    lit_lens[144..256].fill(9);
    lit_lens[256..280].fill(7);
    lit_lens[280..288].fill(8);
    let dist_lens = [5u8; 32];
    Ok((
        HuffmanTable::from_lengths(&lit_lens)?,
        HuffmanTable::from_lengths(&dist_lens)?,
    ))
}

fn dynamic_tables(br: &mut BitReader<'_>) -> Result<(HuffmanTable, HuffmanTable), String> {
    const CL_ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let hlit = br.read_bits(5)? as usize + 257;
    let hdist = br.read_bits(5)? as usize + 1;
    let hclen = br.read_bits(4)? as usize + 4;
    if hlit > 286 || hdist > 32 || hclen > 19 {
        return Err("Invalid dynamic Huffman header counts".to_string());
    }
    let mut cl_lens = [0u8; 19];
    for i in 0..hclen {
        cl_lens[CL_ORDER[i]] = br.read_bits(3)? as u8;
    }
    let cl_table = HuffmanTable::from_lengths(&cl_lens)?;
    let total = hlit + hdist;
    let mut lengths = vec![0u8; total];
    let mut idx = 0usize;
    while idx < total {
        let sym = cl_table.decode(br)?;
        match sym {
            0..=15 => {
                lengths[idx] = sym as u8;
                idx += 1;
            }
            16 => {
                if idx == 0 {
                    return Err("Invalid repeat code at start of Huffman lengths".to_string());
                }
                let prev = lengths[idx - 1];
                let rep = br.read_bits(2)? as usize + 3;
                if idx + rep > total {
                    return Err("Huffman length repeat overflows table".to_string());
                }
                lengths[idx..idx + rep].fill(prev);
                idx += rep;
            }
            17 => {
                let rep = br.read_bits(3)? as usize + 3;
                if idx + rep > total {
                    return Err("Huffman zero repeat (17) overflows table".to_string());
                }
                lengths[idx..idx + rep].fill(0);
                idx += rep;
            }
            18 => {
                let rep = br.read_bits(7)? as usize + 11;
                if idx + rep > total {
                    return Err("Huffman zero repeat (18) overflows table".to_string());
                }
                lengths[idx..idx + rep].fill(0);
                idx += rep;
            }
            _ => return Err("Invalid code-length symbol".to_string()),
        }
    }
    let lit = HuffmanTable::from_lengths(&lengths[..hlit])?;
    let dist = HuffmanTable::from_lengths(&lengths[hlit..])?;
    Ok((lit, dist))
}

/// Decompresses raw RFC 1951 DEFLATE stream enforcing a strict `max_out` ceiling.
pub fn inflate_raw(src: &[u8], max_out: usize) -> Result<Vec<u8>, String> {
    const LEN_BASE: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];

    let mut br = BitReader::new(src);
    let mut out = Vec::with_capacity(src.len().saturating_mul(2).min(max_out));

    loop {
        let bfinal = br.read_bits(1)?;
        let btype = br.read_bits(2)?;
        match btype {
            0 => {
                br.align_byte_slice();
                let hdr = br.read_aligned_bytes(4)?;
                let len = u16::from_le_bytes([hdr[0], hdr[1]]);
                let nlen = u16::from_le_bytes([hdr[2], hdr[3]]);
                if len != !nlen {
                    return Err("Corrupt stored DEFLATE block length".to_string());
                }
                let ulen = len as usize;
                if out.len().saturating_add(ulen) > max_out {
                    return Err("Decompressed entry exceeds size limit".to_string());
                }
                let payload = br.read_aligned_bytes(ulen)?;
                out.extend_from_slice(payload);
            }
            1 | 2 => {
                let (lit_tbl, dist_tbl) = if btype == 1 {
                    fixed_tables()?
                } else {
                    dynamic_tables(&mut br)?
                };
                loop {
                    let sym = lit_tbl.decode(&mut br)? as usize;
                    if sym < 256 {
                        if out.len() >= max_out {
                            return Err("Decompressed entry exceeds size limit".to_string());
                        }
                        out.push(sym as u8);
                    } else if sym == 256 {
                        break;
                    } else {
                        let l_idx = sym - 257;
                        if l_idx >= LEN_BASE.len() {
                            return Err("Invalid DEFLATE length symbol".to_string());
                        }
                        let length = LEN_BASE[l_idx] + br.read_bits(LEN_EXTRA[l_idx])? as usize;
                        let d_sym = dist_tbl.decode(&mut br)? as usize;
                        if d_sym >= DIST_BASE.len() {
                            return Err("Invalid DEFLATE distance symbol".to_string());
                        }
                        let dist = DIST_BASE[d_sym] + br.read_bits(DIST_EXTRA[d_sym])? as usize;
                        if dist == 0 || dist > out.len() {
                            return Err("Invalid DEFLATE back-reference distance".to_string());
                        }
                        if out.len().saturating_add(length) > max_out {
                            return Err("Decompressed entry exceeds size limit".to_string());
                        }
                        out.reserve(length);
                        for _ in 0..length {
                            let b = out[out.len() - dist];
                            out.push(b);
                        }
                    }
                }
            }
            _ => return Err("Invalid DEFLATE block type".to_string()),
        }
        if bfinal != 0 {
            break;
        }
    }
    Ok(out)
}

#[inline]
fn le_u16(b: &[u8], off: usize) -> Result<u16, String> {
    b.get(off..off + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| "Truncated ZIP structure".to_string())
}

#[inline]
fn le_u32(b: &[u8], off: usize) -> Result<u32, String> {
    b.get(off..off + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| "Truncated ZIP structure".to_string())
}

fn sanitize_zip_rel_path(raw: &str) -> Result<Option<String>, String> {
    let norm = raw.replace('\\', "/");
    if norm.is_empty() {
        return Ok(None);
    }
    if norm.ends_with('/') {
        // Still reject traversal directory names before skipping.
        for seg in norm.trim_end_matches('/').split('/') {
            if seg == ".." || seg.contains('\0') || seg.contains(':') {
                return Err(format!("Unsafe path in archive: {}", raw));
            }
        }
        return Ok(None);
    }
    if norm.starts_with('/') || norm.contains('\0') || norm.contains(':') {
        return Err(format!("Unsafe absolute or drive path in archive: {}", raw));
    }
    let mut clean_parts = Vec::new();
    for seg in norm.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err(format!("Path traversal rejected in archive: {}", raw));
        }
        clean_parts.push(seg);
    }
    if clean_parts.is_empty() {
        return Ok(None);
    }
    Ok(Some(clean_parts.join("/")))
}

/// Safely unpacks a ZIP archive (Stored & DEFLATE) into `dest_dir`, blocking zip-slip and bombs.
pub fn extract_zip_safe(zip_bytes: &[u8], dest_dir: &Path) -> Result<usize, String> {
    const MAX_ENTRIES: usize = 4096;
    const MAX_FILE_BYTES: usize = 48 * 1024 * 1024;
    const MAX_TOTAL_BYTES: usize = 128 * 1024 * 1024;

    if zip_bytes.len() < 22 {
        return Err("Archive is too small to be a ZIP file".to_string());
    }
    let min_eocd_pos = zip_bytes.len().saturating_sub(22 + 65535);
    let mut eocd_off = None;
    let mut idx = zip_bytes.len() - 22;
    loop {
        if &zip_bytes[idx..idx + 4] == b"PK\x05\x06" {
            let comment_len = le_u16(zip_bytes, idx + 20)? as usize;
            if idx + 22 + comment_len == zip_bytes.len() {
                eocd_off = Some(idx);
                break;
            }
        }
        if idx == min_eocd_pos {
            break;
        }
        idx -= 1;
    }
    let eocd = eocd_off.ok_or_else(|| "ZIP End of Central Directory not found".to_string())?;
    let total_entries = le_u16(zip_bytes, eocd + 10)? as usize;
    let cd_size = le_u32(zip_bytes, eocd + 12)? as usize;
    let cd_offset = le_u32(zip_bytes, eocd + 16)? as usize;
    if total_entries == 0xFFFF || cd_size == 0xFFFFFFFF || cd_offset == 0xFFFFFFFF {
        return Err("ZIP64 archives are not supported for browser extensions".to_string());
    }
    if total_entries > MAX_ENTRIES {
        return Err(format!("Too many files in extension archive ({})", total_entries));
    }
    let cd_end = cd_offset
        .checked_add(cd_size)
        .ok_or_else(|| "ZIP central directory offset overflow".to_string())?;
    if cd_end > eocd {
        return Err("Corrupt ZIP central directory bounds".to_string());
    }

    fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create extension dir: {}", e))?;

    let mut pos = cd_offset;
    let mut total_unpacked: usize = 0;
    let mut extracted_files: usize = 0;

    for _ in 0..total_entries {
        if pos + 46 > cd_end || &zip_bytes[pos..pos + 4] != b"PK\x01\x02" {
            return Err("Invalid ZIP central directory entry".to_string());
        }
        let method = le_u16(zip_bytes, pos + 10)?;
        let comp_size = le_u32(zip_bytes, pos + 20)? as usize;
        let uncomp_size = le_u32(zip_bytes, pos + 24)? as usize;
        let name_len = le_u16(zip_bytes, pos + 28)? as usize;
        let extra_len = le_u16(zip_bytes, pos + 30)? as usize;
        let comment_len = le_u16(zip_bytes, pos + 32)? as usize;
        let ext_attrs = le_u32(zip_bytes, pos + 38)?;
        let local_off = le_u32(zip_bytes, pos + 42)? as usize;

        let name_start = pos + 46;
        let name_end = name_start + name_len;
        let next_pos = name_end + extra_len + comment_len;
        if next_pos > cd_end {
            return Err("ZIP entry extends past central directory".to_string());
        }
        let raw_name = std::str::from_utf8(&zip_bytes[name_start..name_end])
            .map_err(|_| "Non-UTF8 filename in archive".to_string())?;
        pos = next_pos;

        // Skip POSIX symlinks safely if encoded in external attributes.
        let unix_mode = (ext_attrs >> 16) & 0o170000;
        if unix_mode == 0o120000 {
            continue;
        }
        let Some(rel_path) = sanitize_zip_rel_path(raw_name)? else {
            continue;
        };
        if uncomp_size > MAX_FILE_BYTES || comp_size > MAX_FILE_BYTES {
            return Err(format!("Extension file {} exceeds 48 MB limit", rel_path));
        }

        if local_off + 30 > zip_bytes.len() || &zip_bytes[local_off..local_off + 4] != b"PK\x03\x04" {
            return Err(format!("Invalid local header for {}", rel_path));
        }
        let l_name_len = le_u16(zip_bytes, local_off + 26)? as usize;
        let l_extra_len = le_u16(zip_bytes, local_off + 28)? as usize;
        let data_start = local_off + 30 + l_name_len + l_extra_len;
        let data_end = data_start
            .checked_add(comp_size)
            .ok_or_else(|| "ZIP payload offset overflow".to_string())?;
        if data_end > cd_offset {
            return Err(format!("ZIP data out of bounds for {}", rel_path));
        }
        let comp_slice = &zip_bytes[data_start..data_end];

        let out_bytes: Vec<u8> = match method {
            0 => {
                if comp_slice.len() > MAX_FILE_BYTES {
                    return Err("Stored entry exceeds size limit".to_string());
                }
                comp_slice.to_vec()
            }
            8 => inflate_raw(comp_slice, MAX_FILE_BYTES)?,
            other => {
                return Err(format!("Unsupported ZIP compression method {} ({})", other, rel_path));
            }
        };

        total_unpacked = total_unpacked
            .checked_add(out_bytes.len())
            .ok_or_else(|| "Total unpacked size overflow".to_string())?;
        if total_unpacked > MAX_TOTAL_BYTES {
            return Err("Unpacked extension exceeds 128 MB limit".to_string());
        }

        let out_path = dest_dir.join(&rel_path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed creating parent dir: {}", e))?;
        }
        fs::write(&out_path, &out_bytes)
            .map_err(|e| format!("Failed writing {}: {}", rel_path, e))?;
        extracted_files += 1;
    }

    unwrap_single_wrapper_dir(dest_dir)?;
    Ok(extracted_files)
}

fn unwrap_single_wrapper_dir(dest_dir: &Path) -> Result<(), String> {
    if dest_dir.join("manifest.json").is_file() {
        return Ok(());
    }
    let entries: Vec<fs::DirEntry> = fs::read_dir(dest_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|e| e.file_name().to_string_lossy() != "__MACOSX")
        .collect();
    if entries.len() != 1 || !entries[0].path().is_dir() {
        return Ok(());
    }
    let wrapper = entries[0].path();
    if !wrapper.join("manifest.json").is_file() {
        return Ok(());
    }
    for child in fs::read_dir(&wrapper).map_err(|e| e.to_string())? {
        let child = child.map_err(|e| e.to_string())?;
        let target = dest_dir.join(child.file_name());
        fs::rename(child.path(), target).map_err(|e| e.to_string())?;
    }
    let _ = fs::remove_dir(&wrapper);
    Ok(())
}

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64_ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64_ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if !clean.len().is_multiple_of(4) {
        return Err("Invalid base64 length".to_string());
    }
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for chunk in clean.as_chunks::<4>().0 {
        let mut vals = [0u8; 4];
        let mut pad = 0usize;
        for (i, &ch) in chunk.iter().enumerate() {
            if ch == b'=' {
                if i < 2 || (i == 2 && chunk[3] != b'=') {
                    return Err("Invalid base64 padding".to_string());
                }
                pad += 1;
                vals[i] = 0;
            } else if pad > 0 {
                return Err("Data after base64 padding".to_string());
            } else {
                vals[i] = match ch {
                    b'A'..=b'Z' => ch - b'A',
                    b'a'..=b'z' => ch - b'a' + 26,
                    b'0'..=b'9' => ch - b'0' + 52,
                    b'+' => 62,
                    b'/' => 63,
                    _ => return Err("Invalid character in base64 data".to_string()),
                };
            }
        }
        let n = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6)
            | (vals[3] as u32);
        out.push(((n >> 16) & 0xFF) as u8);
        if pad < 2 {
            out.push(((n >> 8) & 0xFF) as u8);
        }
        if pad < 1 {
            out.push((n & 0xFF) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_zip(entries: &[(&str, u16, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut cd = Vec::new();
        for &(name, method, data) in entries {
            let local_off = out.len() as u32;
            let name_bytes = name.as_bytes();
            let sz = data.len() as u32;
            out.extend_from_slice(b"PK\x03\x04");
            out.extend_from_slice(&20u16.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&method.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
            out.extend_from_slice(&sz.to_le_bytes());
            out.extend_from_slice(&sz.to_le_bytes());
            out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name_bytes);
            out.extend_from_slice(data);

            cd.extend_from_slice(b"PK\x01\x02");
            cd.extend_from_slice(&20u16.to_le_bytes());
            cd.extend_from_slice(&20u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&method.to_le_bytes());
            cd.extend_from_slice(&0u32.to_le_bytes());
            cd.extend_from_slice(&0u32.to_le_bytes());
            cd.extend_from_slice(&sz.to_le_bytes());
            cd.extend_from_slice(&sz.to_le_bytes());
            cd.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u16.to_le_bytes());
            cd.extend_from_slice(&0u32.to_le_bytes());
            cd.extend_from_slice(&local_off.to_le_bytes());
            cd.extend_from_slice(name_bytes);
        }
        let cd_off = out.len() as u32;
        let cd_size = cd.len() as u32;
        out.extend_from_slice(&cd);
        out.extend_from_slice(b"PK\x05\x06");
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_size.to_le_bytes());
        out.extend_from_slice(&cd_off.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    #[test]
    fn store_input_parsing_supports_chrome_edge_and_raw_ids() {
        let id = "cjpalhdlnbpafiamejdnhcphjbkeiagm";
        assert_eq!(parse_store_input(id), Some((id.to_string(), StoreSource::Chrome)));
        assert_eq!(
            parse_store_input("https://chromewebstore.google.com/detail/ublock-origin/cjpalhdlnbpafiamejdnhcphjbkeiagm?hl=zh-CN"),
            Some((id.to_string(), StoreSource::Chrome))
        );
        assert_eq!(
            parse_store_input("https://chrome.google.com/webstore/detail/cjpalhdlnbpafiamejdnhcphjbkeiagm"),
            Some((id.to_string(), StoreSource::Chrome))
        );
        assert_eq!(
            parse_store_input("https://microsoftedge.microsoft.com/addons/detail/dark-reader/ifoakfbpdcdoeenechcleahebpibofpc"),
            Some(("ifoakfbpdcdoeenechcleahebpibofpc".to_string(), StoreSource::Edge))
        );
        assert_eq!(parse_store_input("https://example.com/not-an-id"), None);
        assert!(!is_valid_chrome_ext_id("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"));
    }

    #[test]
    fn strip_crx_header_handles_crx3_crx2_and_zip() {
        let zip = b"PK\x03\x04dummy-zip";
        assert_eq!(strip_crx_header(zip).unwrap(), zip);

        let mut crx3 = Vec::new();
        crx3.extend_from_slice(b"Cr24");
        crx3.extend_from_slice(&3u32.to_le_bytes());
        crx3.extend_from_slice(&6u32.to_le_bytes());
        crx3.extend_from_slice(b"header");
        crx3.extend_from_slice(zip);
        assert_eq!(strip_crx_header(&crx3).unwrap(), zip);

        let mut crx2 = Vec::new();
        crx2.extend_from_slice(b"Cr24");
        crx2.extend_from_slice(&2u32.to_le_bytes());
        crx2.extend_from_slice(&3u32.to_le_bytes());
        crx2.extend_from_slice(&4u32.to_le_bytes());
        crx2.extend_from_slice(b"key");
        crx2.extend_from_slice(b"sign");
        crx2.extend_from_slice(zip);
        assert_eq!(strip_crx_header(&crx2).unwrap(), zip);
    }

    #[test]
    fn inflate_raw_decodes_stored_and_fixed_overlapping_lz77() {
        // Stored block: "Hi"
        let stored = [0x01u8, 0x02, 0x00, 0xFD, 0xFF, b'H', b'i'];
        assert_eq!(inflate_raw(&stored, 64).unwrap(), b"Hi");

        // Fixed Huffman compressed "aaaaaaaaaa" (dist=1 overlapping copy):
        // 'a' (97): 0x30+97 = 0x91 = 0b10010001 -> reversed bits: 10001001
        // len=9 -> sym 263 (0000111 -> rev 1110000), dist=1 -> code 00000, EOB 256 -> 0000000
        let mut bits: Vec<u8> = Vec::new();
        fn push_bits(bits: &mut Vec<u8>, val: u32, n: u8) {
            for i in 0..n {
                bits.push(((val >> i) & 1) as u8);
            }
        }
        fn push_huff(bits: &mut Vec<u8>, code: u32, n: u8) {
            for i in (0..n).rev() {
                bits.push(((code >> i) & 1) as u8);
            }
        }
        push_bits(&mut bits, 1, 1); // BFINAL
        push_bits(&mut bits, 1, 2); // BTYPE = fixed
        push_huff(&mut bits, 0x30 + b'a' as u32, 8);
        push_huff(&mut bits, 263 - 256, 7); // len = 9
        push_huff(&mut bits, 0, 5); // dist = 1
        push_huff(&mut bits, 0, 7); // EOB 256
        let mut packed = vec![0u8; bits.len().div_ceil(8)];
        for (i, &b) in bits.iter().enumerate() {
            if b != 0 {
                packed[i / 8] |= 1 << (i % 8);
            }
        }
        assert_eq!(inflate_raw(&packed, 64).unwrap(), b"aaaaaaaaaa");
        assert!(inflate_raw(&packed, 5).is_err());
    }

    #[test]
    fn extract_zip_safe_unwraps_wrapper_and_blocks_traversal() {
        let dir = std::env::temp_dir().join("fynexo_crx_zip_test");
        let _ = fs::remove_dir_all(&dir);

        let wrapped = build_test_zip(&[
            ("myext/manifest.json", 0, br#"{"name":"Test","manifest_version":3}"#),
            ("myext/bg.js", 0, b"console.log(1);"),
        ]);
        let count = extract_zip_safe(&wrapped, &dir).unwrap();
        assert_eq!(count, 2);
        assert!(dir.join("manifest.json").is_file());
        assert!(dir.join("bg.js").is_file());
        let _ = fs::remove_dir_all(&dir);

        let bad = build_test_zip(&[("../evil.txt", 0, b"nope")]);
        assert!(extract_zip_safe(&bad, &dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn inflate_raw_decodes_dynamic_huffman_and_method8_zip() {
        const DYNAMIC_COMP: [u8; 96] = [
            0xED, 0xCB, 0x3D, 0x0A, 0x80, 0x30, 0x0C, 0x06, 0xD0, 0xAB, 0xC4, 0xCC, 0x6E, 0x6E,
            0x5D, 0x45, 0xF1, 0x00, 0xEE, 0x52, 0x34, 0xC1, 0x80, 0x6D, 0xA1, 0xAD, 0x52, 0x11,
            0xEF, 0x6E, 0xBC, 0x82, 0xB3, 0xE3, 0xF7, 0xF3, 0xFA, 0xD3, 0x53, 0x09, 0xD0, 0xAE,
            0x31, 0x38, 0x82, 0xAE, 0x64, 0xF2, 0x49, 0x82, 0x87, 0xE5, 0xF4, 0xD6, 0xC9, 0x0C,
            0xC3, 0xCE, 0xEC, 0xAC, 0x66, 0xE2, 0xCD, 0x66, 0x82, 0x83, 0xA2, 0xB0, 0xCC, 0x36,
            0xEB, 0xA9, 0x82, 0xFE, 0xD7, 0x1F, 0xF4, 0x85, 0xBA, 0x09, 0x53, 0xCA, 0x93, 0x0E,
            0x2F, 0x44, 0xD3, 0xD4, 0xA8, 0x94, 0xD0, 0xE0, 0xA8, 0x3D, 0xDE, 0x0F,
        ];
        let expected = format!(
            "{}{}",
            "Fynexo Chrome Extension dynamic Huffman deflate verification! ".repeat(6),
            r#"{"manifest_version":3,"name":"Test"}"#
        );
        let inflated = inflate_raw(&DYNAMIC_COMP, 4096).unwrap();
        assert_eq!(String::from_utf8(inflated).unwrap(), expected);

        let dir = std::env::temp_dir().join("fynexo_crx_dyn_zip_test");
        let _ = fs::remove_dir_all(&dir);
        let zip_m8 = build_test_zip(&[("manifest.json", 8, &DYNAMIC_COMP)]);
        assert_eq!(extract_zip_safe(&zip_m8, &dir).unwrap(), 1);
        assert_eq!(
            fs::read_to_string(dir.join("manifest.json")).unwrap(),
            expected
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn base64_roundtrip_works() {
        for raw in [&b""[..], b"f", b"fn", b"fynexo", b"\x00\xff\x80\x7f"] {
            let enc = base64_encode(raw);
            assert_eq!(base64_decode(&enc).unwrap(), raw);
        }
    }

    fn pb_len_field(field_num: u64, wire_type: u64, payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut tag = (field_num << 3) | wire_type;
        loop {
            let b = (tag & 0x7f) as u8;
            tag >>= 7;
            if tag == 0 {
                out.push(b);
                break;
            } else {
                out.push(b | 0x80);
            }
        }
        let mut len = payload.len() as u64;
        loop {
            let b = (len & 0x7f) as u8;
            len >>= 7;
            if len == 0 {
                out.push(b);
                break;
            } else {
                out.push(b | 0x80);
            }
        }
        out.extend_from_slice(payload);
        out
    }

    #[test]
    fn sha256_and_crx3_proof_verification_work() {
        assert_eq!(
            sha256(b"abc"),
            [
                0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
                0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
                0xf2, 0x00, 0x15, 0xad
            ]
        );

        let pubkey = b"fake-spki-der-public-key-for-fynexo-test";
        let expected_id = ext_id_from_public_key(pubkey);
        assert!(is_valid_chrome_ext_id(&expected_id));

        let digest = sha256(pubkey);
        let proof = [
            pb_len_field(1, 2, pubkey),
            pb_len_field(2, 2, b"rsa-signature-bytes"),
        ]
        .concat();
        let signed_data = pb_len_field(1, 2, &digest[..16]);
        let hdr = [
            pb_len_field(2, 2, &proof),
            pb_len_field(10000, 2, &signed_data),
        ]
        .concat();

        let zip = b"PK\x03\x04dummy-zip";
        let mut crx3 = Vec::new();
        crx3.extend_from_slice(b"Cr24");
        crx3.extend_from_slice(&3u32.to_le_bytes());
        crx3.extend_from_slice(&(hdr.len() as u32).to_le_bytes());
        crx3.extend_from_slice(&hdr);
        crx3.extend_from_slice(zip);

        let (stripped, detected) = verify_and_strip_crx(&crx3, Some(&expected_id), true).unwrap();
        assert_eq!(stripped, zip);
        assert_eq!(detected.as_deref(), Some(expected_id.as_str()));

        let wrong_id = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert!(verify_and_strip_crx(&crx3, Some(wrong_id), true).is_err());
        assert!(verify_and_strip_crx(zip, Some(&expected_id), true).is_err());
        assert!(verify_and_strip_crx(zip, Some(&expected_id), false).is_ok());
    }
}
