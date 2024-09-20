pub struct Ethernet<P = ()> {
    slice: P,
    size: EtherSize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum EtherType {
    IPv4 = 0x0800,
    IPv6 = 0x86dd,
    Arp = 0x0806,
    WakeOnLan = 0x0842,
    VlanTaggedFrame = 0x8100,
    ProviderBridging = 0x88A8,
    VlanDoubleTaggedFrame = 0x9100,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(usize)]
pub enum EtherSize {
    S18 = 18,
    S16 = 16,
    S14 = 14,
}

impl From<EtherType> for u16 {
    fn from(value: EtherType) -> Self {
        match value {
            EtherType::IPv4 => 0x0800,
            EtherType::IPv6 => 0x86dd,
            EtherType::Arp => 0x0806,
            EtherType::WakeOnLan => 0x0842,
            EtherType::VlanTaggedFrame => 0x8100,
            EtherType::ProviderBridging => 0x88A8,
            EtherType::VlanDoubleTaggedFrame => 0x9100,
            EtherType::Other(v) => v,
        }
    }
}

impl From<[u8; 2]> for EtherType {
    fn from(value: [u8; 2]) -> Self {
        let num = u16::from_be_bytes(value);
        EtherType::from(num)
    }
}

impl TryFrom<&[u8]> for EtherType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 2 {
            return Err(());
        }

        let array = *value.first_chunk::<2>().unwrap();
        Ok(EtherType::from(array))
    }
}

impl From<u16> for EtherType {
    fn from(value: u16) -> Self {
        match value {
            0x0800 => Self::IPv4,
            0x86dd => Self::IPv6,
            0x0806 => Self::Arp,
            0x0842 => Self::WakeOnLan,
            0x8100 => Self::VlanTaggedFrame,
            0x88A8 => Self::ProviderBridging,
            0x9100 => Self::VlanDoubleTaggedFrame,
            x => Self::Other(x),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    WrongSize(usize),
    WrongSizeForType(EtherType, usize),
}

impl<P: AsMut<[u8]> + AsRef<[u8]>> Ethernet<P> {
    pub fn set_destination(&mut self, new_dest: &[u8; 6]) {
        self.slice.as_mut()[0..6].copy_from_slice(new_dest);
    }

    pub fn set_source(&mut self, new_dest: &[u8; 6]) {
        self.slice.as_mut()[6..12].copy_from_slice(new_dest);
    }

    pub fn set_ethertype(&mut self, ethertype: EtherType) {
        self.slice.as_mut()[self.size as usize - 2..self.size as usize]
            .copy_from_slice(&u16::from(ethertype).to_be_bytes());
    }

    pub fn slice_mut(&mut self) -> &mut [u8] {
        self.slice.as_mut()
    }
}

impl Ethernet<()> {
    pub const MIN_LEN: usize = 14;
    pub const MAX_LEN: usize = 18;
}

impl<'pkt> Ethernet<&'pkt [u8]> {
    pub fn new(slice: &'pkt [u8]) -> Result<(Ethernet<&'pkt [u8]>, &'pkt [u8]), Error> {
        if slice.len() < Ethernet::MIN_LEN {
            return Err(Error::WrongSize(slice.len()));
        }

        let size = match EtherType::from(*slice[12..14].first_chunk::<2>().unwrap()) {
            EtherType::VlanDoubleTaggedFrame if slice.len() >= Ethernet::MAX_LEN => EtherSize::S18,
            EtherType::VlanTaggedFrame if slice.len() >= Ethernet::MIN_LEN + 2 => EtherSize::S16,
            EtherType::Other(_) if slice.len() >= Ethernet::MIN_LEN => EtherSize::S14,
            _ if slice.len() >= Ethernet::MIN_LEN => EtherSize::S14,
            x => return Err(Error::WrongSizeForType(x, slice.len())),
        };

        let (parsed, rem) = slice.split_at(size as usize);
        Ok((
            Ethernet {
                slice: parsed,
                size,
            },
            rem,
        ))
    }
}

impl<'pkt> Ethernet<&'pkt mut [u8]> {
    pub fn new_mut(
        slice: &'pkt mut [u8],
    ) -> Result<(Ethernet<&'pkt mut [u8]>, &'pkt mut [u8]), Error> {
        if slice.len() < Ethernet::MIN_LEN {
            return Err(Error::WrongSize(slice.len()));
        }

        let size = match EtherType::from(*slice[12..14].first_chunk::<2>().unwrap()) {
            EtherType::VlanDoubleTaggedFrame if slice.len() >= Ethernet::MAX_LEN => EtherSize::S18,
            EtherType::VlanTaggedFrame if slice.len() >= Ethernet::MIN_LEN + 2 => EtherSize::S16,
            EtherType::Other(_) if slice.len() >= Ethernet::MIN_LEN => EtherSize::S14,
            _ if slice.len() >= Ethernet::MIN_LEN => EtherSize::S14,
            x => return Err(Error::WrongSizeForType(x, slice.len())),
        };

        let (parsed, rem) = slice.split_at_mut(size as usize);
        Ok((
            Ethernet {
                slice: parsed,
                size,
            },
            rem,
        ))
    }
}

impl<P: AsRef<[u8]>> Ethernet<P> {
    pub fn ethertype(&self) -> EtherType {
        match self.size {
            EtherSize::S18 => {
                EtherType::from(*self.slice.as_ref()[16..18].first_chunk::<2>().unwrap())
            }
            EtherSize::S16 => {
                EtherType::from(*self.slice.as_ref()[14..16].first_chunk::<2>().unwrap())
            }
            EtherSize::S14 => {
                EtherType::from(*self.slice.as_ref()[12..14].first_chunk::<2>().unwrap())
            }
        }
    }

    pub fn destination(&self) -> &[u8; 6] {
        self.slice.as_ref()[0..6].try_into().unwrap()
    }

    pub fn slice(&self) -> &[u8] {
        self.slice.as_ref()
    }

    pub fn size_usize(&self) -> usize {
        match self.size {
            EtherSize::S18 => 18,
            EtherSize::S16 => 16,
            EtherSize::S14 => 14,
        }
    }

    pub fn size(&self) -> EtherSize {
        self.size
    }

    pub fn source(&self) -> &[u8; 6] {
        self.slice.as_ref()[6..12].try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::link::eth::EtherSize;
    use crate::link::{EtherType, Ethernet};

    #[test]
    fn create_mut() {
        let mut packet = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x08, 0x00,
        ];
        let (mut eth, rem) = Ethernet::new_mut(&mut packet).unwrap();

        assert_eq!(rem.len(), 0);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::IPv4);

        eth.set_source(&[0x02, 0x02, 0x02, 0x02, 0x02, 0x02]);
        eth.set_destination(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        assert_eq!(eth.source(), &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02]);
        assert_eq!(eth.destination(), &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    }

    #[test]
    fn create_ref() {
        let packet = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x08, 0x00,
        ];
        let (eth, rem) = Ethernet::new(&packet).unwrap();

        assert_eq!(rem.len(), 0);
        assert_eq!(eth.size, EtherSize::S14);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::IPv4);
    }

    #[test]
    fn vlan_tagged() {
        let packet = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x81, 0x00,
            0x08, 0x00,
        ];
        let (eth, rem) = Ethernet::new(&packet).unwrap();

        assert_eq!(rem.len(), 0);
        assert_eq!(eth.size, EtherSize::S16);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::IPv4);
    }

    #[test]
    fn double_vlan_tagged() {
        let packet = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x91, 0x00,
            0x81, 0x00, 0x08, 0x00,
        ];
        let (eth, rem) = Ethernet::new(&packet).unwrap();

        assert_eq!(rem.len(), 0);
        assert_eq!(eth.size, EtherSize::S18);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::IPv4);
    }

    #[test]
    fn change_ehertype() {
        let mut packet = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x81, 0x00,
            0x08, 0x00,
        ];
        let (mut eth, rem) = Ethernet::new_mut(&mut packet).unwrap();

        assert_eq!(rem.len(), 0);
        assert_eq!(eth.size, EtherSize::S16);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::IPv4);

        eth.set_ethertype(EtherType::Arp);

        assert_eq!(eth.size, EtherSize::S16);
        assert_eq!(eth.destination(), &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(eth.source(), &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01]);
        assert_eq!(eth.ethertype(), EtherType::Arp);
    }
}
