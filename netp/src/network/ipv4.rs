use super::ipnum::InetProtocol;

pub struct IPv4<P = ()> {
    slice: P,
    size: IPv4Size,
}

impl<'pkt> IPv4<&'pkt [u8]> {
    pub fn new(slice: &'pkt [u8]) -> Result<(Self, &'pkt [u8]), Error> {
        if slice.len() < IPv4::MIN_LEN {
            return Err(Error::InvalidSize(slice.len()));
        }

        let size = IPv4Size::try_from_ihl_u8(slice[0] & 0xF).map_err(Error::InvalidIhl)?;

        if slice[0] >> 4 != 4 {
            return Err(Error::InvalidVersion(slice[0] >> 4));
        }

        if slice.len() < size as usize {
            return Err(Error::InvalidSizeForIhl(slice.len(), size));
        }

        let (slice, rem) = slice.split_at(size as usize);
        Ok((Self { slice, size }, rem))
    }
}

impl<'pkt> IPv4<&'pkt mut [u8]> {
    pub fn new_mut(slice: &'pkt mut [u8]) -> Result<(Self, &'pkt mut [u8]), Error> {
        if slice.len() < IPv4::MIN_LEN {
            return Err(Error::InvalidSize(slice.len()));
        }

        let size = IPv4Size::try_from_ihl_u8(slice[0] & 0xF).map_err(Error::InvalidIhl)?;

        if slice[0] >> 4 != 4 {
            return Err(Error::InvalidVersion(slice[0] >> 4));
        }

        if slice.len() < size as usize {
            return Err(Error::InvalidSizeForIhl(slice.len(), size));
        }

        let (slice, rem) = slice.split_at_mut(size as usize);
        Ok((Self { slice, size }, rem))
    }
}

#[derive(Debug)]
pub enum IhlError {
    InvalidIhl(u8),
}

pub enum Error {
    InvalidIhl(IhlError),
    InvalidSize(usize),
    InvalidSizeForIhl(usize, IPv4Size),
    InvalidVersion(u8),
}

impl<P: AsMut<[u8]> + AsRef<[u8]>> IPv4<P> {
    pub fn set_csum(&mut self, csum: u16) {
        self.slice.as_mut()[10..12].copy_from_slice(&csum.to_be_bytes());
    }

    pub fn update_csum(&mut self) {
        self.set_csum(self.calc_csum())
    }

    pub fn slice_mut(&mut self) -> &mut [u8] {
        self.slice.as_mut()
    }

    pub fn set_source(&mut self, source: &[u8; 4]) {
        self.slice.as_mut()[12..16].copy_from_slice(source)
    }

    pub fn set_source_u32(&mut self, source: u32) {
        self.slice.as_mut()[12..16].copy_from_slice(&source.to_be_bytes())
    }

    pub fn set_destination_u32(&mut self, destination: u32) {
        self.slice.as_mut()[16..20].copy_from_slice(&destination.to_be_bytes())
    }

    pub fn set_destination(&mut self, destination: &[u8; 4]) {
        self.slice.as_mut()[16..20].copy_from_slice(destination)
    }

    pub fn set_total_length(&mut self, value: &[u8; 2]) {
        self.slice.as_mut()[2..4].copy_from_slice(value)
    }

    pub fn set_total_length_u16(&mut self, value: u16) {
        self.slice.as_mut()[2..4].copy_from_slice(&value.to_be_bytes())
    }

    pub fn set_protocol(&mut self, protocol: InetProtocol) {
        self.slice.as_mut()[9] = u8::from(protocol);
    }
}

impl IPv4<()> {
    pub const MIN_LEN: usize = 20;
    pub const MAX_LEN: usize = 60;
}

impl<P: AsRef<[u8]>> IPv4<P> {
    pub fn csum(&self) -> u16 {
        u16::from_be_bytes(*self.slice.as_ref()[10..12].first_chunk::<2>().unwrap())
    }

    pub fn calc_csum(&self) -> u16 {
        etherparse::checksum::Sum16BitWords::new()
            .add_2bytes([(4 << 4) | self.ihl_u8(), (self.dscp() << 2) | self.ecn()])
            .add_2bytes(self.total_length().to_be_bytes())
            .add_2bytes(self.identification().to_be_bytes())
            .add_2bytes({
                let frag_off_be = self.fragment_offset();
                let flags = {
                    let mut result = 0;
                    if self.dont_fragment() {
                        result |= 64;
                    }
                    if self.more_fragments() {
                        result |= 32;
                    }
                    result
                };
                [flags | (frag_off_be[0] & 0x1f), frag_off_be[1]]
            })
            .add_2bytes([self.ttl(), self.protocol_u8()])
            .add_4bytes(*self.source())
            .add_4bytes(*self.destination())
            .add_slice(self.options())
            .ones_complement()
            .to_be()
    }

    pub fn slice(&self) -> &[u8] {
        &self.slice.as_ref()
    }

    pub fn version(&self) -> u8 {
        self.slice.as_ref()[0] >> 4
    }

    pub fn source(&self) -> &[u8; 4] {
        self.slice.as_ref()[12..16].first_chunk::<4>().unwrap()
    }

    pub fn source_u32(&self) -> u32 {
        u32::from_be_bytes(*self.slice.as_ref()[12..16].first_chunk::<4>().unwrap())
    }

    pub fn destination_u32(&self) -> u32 {
        u32::from_be_bytes(*self.slice.as_ref()[16..20].first_chunk::<4>().unwrap())
    }

    pub fn destination(&self) -> &[u8; 4] {
        self.slice.as_ref()[16..20].first_chunk::<4>().unwrap()
    }

    pub fn ttl(&self) -> u8 {
        self.slice.as_ref()[8]
    }

    pub fn options(&self) -> &[u8] {
        &self.slice.as_ref()[IPv4::MIN_LEN..self.size as usize]
    }

    pub fn dscp(&self) -> u8 {
        self.slice.as_ref()[1] >> 2
    }

    pub fn ecn(&self) -> u8 {
        self.slice.as_ref()[1] & 0b11
    }

    pub fn total_length(&self) -> u16 {
        u16::from_be_bytes(*self.slice.as_ref()[2..4].first_chunk::<2>().unwrap())
    }

    pub fn identification(&self) -> u16 {
        u16::from_be_bytes(*self.slice.as_ref()[4..6].first_chunk::<2>().unwrap())
    }

    pub fn total_length_u16(&self) -> u16 {
        u16::from_be_bytes(*self.slice.as_ref()[2..4].first_chunk::<2>().unwrap())
    }

    pub fn fragment_offset(&self) -> [u8; 2] {
        let mut res = [0, 0];
        res[0] = self.slice.as_ref()[6] & 0b11111;
        res[1] = self.slice.as_ref()[7];
        res
    }

    pub fn dont_fragment(&self) -> bool {
        (self.slice.as_ref()[6] >> 6) & 0b01 == 1
    }

    pub fn more_fragments(&self) -> bool {
        (self.slice.as_ref()[6] >> 5) & 0b001 == 1
    }

    pub fn protocol(&self) -> InetProtocol {
        InetProtocol::from(self.slice.as_ref()[9])
    }

    pub fn protocol_u8(&self) -> u8 {
        self.slice.as_ref()[9]
    }

    pub fn ihl_u8(&self) -> u8 {
        self.slice.as_ref()[0] & 0xF
    }

    pub fn size(&self) -> IPv4Size {
        self.size
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(usize)]
pub enum IPv4Size {
    S20 = 20,
    S24 = 24,
    S28 = 28,
    S32 = 32,
    S36 = 36,
    S40 = 40,
    S44 = 44,
    S48 = 48,
    S52 = 52,
    S56 = 56,
    S60 = 60,
}

impl From<IhlError> for Error {
    fn from(value: IhlError) -> Self {
        Error::InvalidIhl(value)
    }
}

impl IPv4Size {
    pub fn try_from_ihl_u8(ihl: u8) -> Result<Self, IhlError> {
        Ok(match ihl {
            5 => IPv4Size::S20,
            6 => IPv4Size::S24,
            7 => IPv4Size::S28,
            8 => IPv4Size::S32,
            9 => IPv4Size::S36,
            10 => IPv4Size::S40,
            11 => IPv4Size::S44,
            12 => IPv4Size::S48,
            13 => IPv4Size::S52,
            14 => IPv4Size::S56,
            15 => IPv4Size::S60,
            x => {
                return Err(IhlError::InvalidIhl(x));
            }
        })
    }
}
