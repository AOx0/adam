pub struct Udp<'pkt> {
    slice: &'pkt mut [u8],
}

pub enum Error {
    InvalidLength(usize),
}

impl<'pkt> Udp<'pkt> {
    pub const SIZE: usize = 8;
    pub fn new(slice: &'pkt mut [u8]) -> Result<(Self, &'pkt mut [u8]), Error> {
        if slice.len() < Self::SIZE {
            return Err(Error::InvalidLength(slice.len()));
        }

        let (slice, rem) = slice.split_at_mut(Self::SIZE);

        Ok((Self { slice }, rem))
    }
}

impl Udp<'_> {
    pub fn source(&self) -> u16 {
        u16::from_be_bytes(*self.slice[0..2].first_chunk::<2>().unwrap())
    }

    pub fn destination(&self) -> u16 {
        u16::from_be_bytes(*self.slice[2..4].first_chunk::<2>().unwrap())
    }

    pub fn length(&self) -> u16 {
        u16::from_be_bytes(*self.slice[4..6].first_chunk::<2>().unwrap())
    }

    pub fn checksum(&self) -> &[u8; 2] {
        self.slice[6..8].first_chunk::<2>().unwrap()
    }

    pub fn set_source(&mut self, source: u16) {
        self.slice[0..2].copy_from_slice(&source.to_be_bytes())
    }

    pub fn set_destination(&mut self, destination: u16) {
        self.slice[2..4].copy_from_slice(&destination.to_be_bytes())
    }

    pub fn set_length(&mut self, length: u16) {
        self.slice[4..6].copy_from_slice(&length.to_be_bytes())
    }

    pub fn set_checksum(&mut self, checksum: u16) {
        self.slice[6..8].copy_from_slice(&checksum.to_be_bytes())
    }

    pub fn set_checksum_zero(&mut self) {
        self.set_checksum(0);
    }
}
