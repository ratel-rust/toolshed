const A__: u16 = 0;
const A00: u16 = 1;
const A01: u16 = 1 << 1;
const A02: u16 = 1 << 2;
const A03: u16 = 1 << 3;
const A04: u16 = 1 << 4;
const A05: u16 = 1 << 5;
const A06: u16 = 1 << 6;
const A07: u16 = 1 << 7;
const A08: u16 = 1 << 8;
const A09: u16 = 1 << 9;
const A10: u16 = 1 << 10;
const A11: u16 = 1 << 11;
const A12: u16 = 1 << 12;
const A13: u16 = 1 << 13;
const A14: u16 = 1 << 14;
const A15: u16 = 1 << 15;

const B__: u32 = 0;
const B00: u32 = 1 << 16;
const B01: u32 = 1 << 17;
const B02: u32 = 1 << 18;
const B03: u32 = 1 << 19;
const B04: u32 = 1 << 20;
const B05: u32 = 1 << 21;
const B06: u32 = 1 << 22;
const B07: u32 = 1 << 23;
const B08: u32 = 1 << 24;
const B09: u32 = 1 << 25;
const B10: u32 = 1 << 26;
const B11: u32 = 1 << 27;
const B12: u32 = 1 << 28;
const B13: u32 = 1 << 29;
const B14: u32 = 1 << 30;
const B15: u32 = 1 << 31;

const C__: u64 = 0;
const C00: u64 = 1 << 32;
const C01: u64 = 1 << 33;
const C02: u64 = 1 << 34;
const C03: u64 = 1 << 35;
const C04: u64 = 1 << 36;
const C05: u64 = 1 << 37;
const C06: u64 = 1 << 38;
const C07: u64 = 1 << 39;
const C08: u64 = 1 << 40;
const C09: u64 = 1 << 41;
const C10: u64 = 1 << 42;
const C11: u64 = 1 << 43;
const C12: u64 = 1 << 44;
const C13: u64 = 1 << 45;
const C14: u64 = 1 << 46;
const C15: u64 = 1 << 47;

static BYTE_MASKS_A: [u16; 256] = [
//    0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F  //
    A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, // 0
    A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, // 1
    A__, A__, A__, A__, A01, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, A__, // 2
    A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A__, A__, A__, A__, A__, A__, // 3
    A__, A12, A13, A14, A15, A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, // 4
    A11, A12, A13, A14, A15, A00, A01, A02, A03, A04, A05, A__, A__, A__, A__, A00, // 5
    A__, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, A00, A01, A02, A03, A04, // 6
    A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, A__, A__, A__, A__, A__, // 7
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // 8
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // 9
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // A
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // B
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // C
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // D
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // E
    A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A10, A11, A12, A13, A14, A15, // F
];

static BYTE_MASKS_B: [u32; 256] = [
//    0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F  //
    B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, // 0
    B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, // 1
    B__, B__, B__, B__, B01, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, B__, // 2
    B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B__, B__, B__, B__, B__, B__, // 3
    B__, B12, B13, B14, B15, B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, // 4
    B11, B12, B13, B14, B15, B00, B01, B02, B03, B04, B05, B__, B__, B__, B__, B00, // 5
    B__, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, B00, B01, B02, B03, B04, // 6
    B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, B__, B__, B__, B__, B__, // 7
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // 8
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // 9
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // A
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // B
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // C
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // D
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // E
    B00, B01, B02, B03, B04, B05, B06, B07, B08, B09, B10, B11, B12, B13, B14, B15, // F
];

static BYTE_MASKS_C: [u64; 256] = [
//    0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F  //
    C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, // 0
    C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, // 1
    C__, C__, C__, C__, C01, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, C__, // 2
    C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C__, C__, C__, C__, C__, C__, // 3
    C__, C12, C13, C14, C15, C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, // 4
    C11, C12, C13, C14, C15, C00, C01, C02, C03, C04, C05, C__, C__, C__, C__, C00, // 5
    C__, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, C00, C01, C02, C03, C04, // 6
    C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, C__, C__, C__, C__, C__, // 7
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // 8
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // 9
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // A
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // B
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // C
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // D
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // E
    C00, C01, C02, C03, C04, C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C15, // F
];

pub trait Bloom {
    fn as_bytes(&self) -> &[u8];

    #[inline]
    fn bloom(&self) -> u64 {
        let s = self.as_bytes();

        match s.len() {
            0 => 0x0001000000000000,

            1 => 0x0002000000000000
               | BYTE_MASKS_A[s[0] as usize] as u64,

            2 => 0x0004000000000000
               | BYTE_MASKS_A[s[0] as usize] as u64
               | BYTE_MASKS_B[s[1] as usize] as u64,

            n => 0x0001000000000000 << n % 16
               | BYTE_MASKS_C[s[2] as usize]
               | BYTE_MASKS_B[s[1] as usize] as u64
               | BYTE_MASKS_A[s[0] as usize] as u64
        }
    }
}

impl Bloom for str {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(self)
    }
}

impl Bloom for [u8] {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl<'a> Bloom for &'a str {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(*self)
    }
}

impl<'a> Bloom for &'a [u8] {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        *self
    }
}
