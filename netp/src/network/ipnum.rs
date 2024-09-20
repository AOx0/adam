#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[repr(u8)]
pub enum InetProtocol {
    /// IPv6 Hop-by-Hop Option \[[RFC8200](https://datatracker.ietf.org/doc/html/rfc8200)\]
    IPV6_HEADER_HOP_BY_HOP = 0,
    /// Internet Control Message \[[RFC792](https://datatracker.ietf.org/doc/html/rfc792)\]
    ICMP = 1,
    /// Internet Group Management \[[RFC1112](https://datatracker.ietf.org/doc/html/rfc1112)\]
    IGMP = 2,
    /// Gateway-to-Gateway \[[RFC823](https://datatracker.ietf.org/doc/html/rfc823)\]
    GGP = 3,
    /// IPv4 encapsulation \[[RFC2003](https://datatracker.ietf.org/doc/html/rfc2003)\]
    IPV4 = 4,
    /// Stream \[[RFC1190](https://datatracker.ietf.org/doc/html/rfc1190)\] \[[RFC1819](https://datatracker.ietf.org/doc/html/rfc1819)\]
    STREAM = 5,
    /// Transmission Control \[[RFC793](https://datatracker.ietf.org/doc/html/rfc793)\]
    TCP = 6,
    /// CBT \[Tony_Ballardie\]
    CBT = 7,
    /// Exterior Gateway Protocol \[[RFC888](https://datatracker.ietf.org/doc/html/rfc888)\] \[David_Mills\]
    EGP = 8,
    /// any private interior gateway (used by Cisco for their IGRP) \[Internet_Assigned_Numbers_Authority\]
    IGP = 9,
    /// BBN RCC Monitoring \[Steve_Chipman\]
    BBN_RCC_MON = 10,
    /// Network Voice Protocol \[[RFC741](https://datatracker.ietf.org/doc/html/rfc741)\]\[Steve_Casner\]
    NVP_II = 11,
    /// PUP
    PUP = 12,
    /// ARGUS (deprecated) \[Robert_W_Scheifler\]
    ARGUS = 13,
    /// EMCON \[mystery contact\]
    EMCON = 14,
    /// Cross Net Debugger \[Haverty, J., "XNET Formats for Internet Protocol Version 4", IEN 158, October 1980.\]\[Jack_Haverty\]
    XNET = 15,
    /// Chaos \[J_Noel_Chiappa\]
    CHAOS = 16,
    /// User Datagram \[[RFC768](https://datatracker.ietf.org/doc/html/rfc768)\]\[Jon_Postel\]
    UDP = 17,
    /// Multiplexing \[Cohen, D. and J. Postel, "Multiplexing Protocol", IEN 90, USC/Information Sciences Institute, May 1979.\]\[Jon_Postel\]
    MUX = 18,
    /// DCN Measurement Subsystems \[David_Mills\]
    DCN_MEAS = 19,
    /// Host Monitoring \[[RFC869](https://datatracker.ietf.org/doc/html/rfc869)\]\[Bob_Hinden\]
    HMP = 20,
    /// Packet Radio Measurement \[Zaw_Sing_Su\]
    PRM = 21,
    /// XEROX NS IDP
    XNS_IDP = 22,
    /// Trunk-1 \[Barry_Boehm\]
    TRUNK1 = 23,
    /// Trunk-2 \[Barry_Boehm\]
    TRUNK2 = 24,
    /// Leaf-1 \[Barry_Boehm\]
    LEAF1 = 25,
    /// Leaf-2 \[Barry_Boehm\]
    LEAF2 = 26,
    /// Reliable Data Protocol \[[RFC908](https://datatracker.ietf.org/doc/html/rfc908)\] \[Bob_Hinden\]
    RDP = 27,
    /// Internet Reliable Transaction \[[RFC938](https://datatracker.ietf.org/doc/html/rfc938)\] \[Trudy_Miller\]
    IRTP = 28,
    /// ISO Transport Protocol Class 4 \[[RFC905](https://datatracker.ietf.org/doc/html/rfc905)\] \[mystery contact\]
    ISO_TP4 = 29,
    /// Bulk Data Transfer Protocol \[[RFC969](https://datatracker.ietf.org/doc/html/rfc969)\] \[David_Clark\]
    NET_BLT = 30,
    /// MFE Network Services Protocol \[Shuttleworth, B., "A Documentary of MFENet, a National Computer Network", UCRL-52317, Lawrence Livermore Labs, Livermore, California, June 1977.\] \[Barry_Howard\]
    MFE_NSP = 31,
    /// MERIT Internodal Protocol \[Hans_Werner_Braun\]
    MERIT_INP = 32,
    /// Datagram Congestion Control Protocol \[[RFC4340](https://datatracker.ietf.org/doc/html/rfc4340)\]
    DCCP = 33,
    /// Third Party Connect Protocol \[Stuart_A_Friedberg\]
    THIRD_PARTY_CONNECT_PROTOCOL = 34,
    /// Inter-Domain Policy Routing Protocol \[Martha_Steenstrup\]
    IDPR = 35,
    /// XTP \[Greg_Chesson\]
    XTP = 36,
    /// Datagram Delivery Protocol \[Wesley_Craig\]
    DDP = 37,
    /// IDPR Control Message Transport Proto \[Martha_Steenstrup\]
    IDPR_CMTP = 38,
    /// TP++ Transport Protocol \[Dirk_Fromhein\]
    TP_PLUS_PLUS = 39,
    /// IL Transport Protocol \[Dave_Presotto\]
    IL = 40,
    /// IPv6 encapsulation \[[RFC2473](https://datatracker.ietf.org/doc/html/rfc2473)\]
    IPV6 = 41,
    /// Source Demand Routing Protocol \[Deborah_Estrin\]
    SDRP = 42,
    /// Routing Header for IPv6 \[Steve_Deering\]
    IPV6_ROUTE_HEADER = 43,
    /// Fragment Header for IPv6 \[Steve_Deering\]
    IPV6_FRAGMENTATION_HEADER = 44,
    /// Inter-Domain Routing Protocol \[Sue_Hares\]
    IDRP = 45,
    /// Reservation Protocol \[[RFC2205](https://datatracker.ietf.org/doc/html/rfc2205)\]\[[RFC3209](https://datatracker.ietf.org/doc/html/rfc3209)\]\[Bob_Braden\]
    RSVP = 46,
    /// Generic Routing Encapsulation \[[RFC2784](https://datatracker.ietf.org/doc/html/rfc2784)\]\[Tony_Li\]
    GRE = 47,
    /// Dynamic Source Routing Protocol \[[RFC4728](https://datatracker.ietf.org/doc/html/rfc4728)\]
    DSR = 48,
    /// BNA \[Gary Salamon\]
    BNA = 49,
    /// Encapsulating Security Payload \[[RFC4303](https://datatracker.ietf.org/doc/html/rfc4303)\]
    ENCAPSULATING_SECURITY_PAYLOAD = 50,
    /// Authentication Header \[[RFC4302](https://datatracker.ietf.org/doc/html/rfc4302)\]
    AUTHENTICATION_HEADER = 51,
    /// Integrated Net Layer Security  TUBA \[K_Robert_Glenn\]
    INLSP = 52,
    /// IP with Encryption (deprecated) \[John_Ioannidis\]
    SWIPE = 53,
    /// NBMA Address Resolution Protocol \[[RFC1735](https://datatracker.ietf.org/doc/html/rfc1735)\]
    NARP = 54,
    /// IP Mobility \[Charlie_Perkins\]
    MOBILE = 55,
    /// Transport Layer Security Protocol using Kryptonet key management \[Christer_Oberg\]
    TLSP = 56,
    /// SKIP \[Tom_Markson\]
    SKIP = 57,
    /// ICMP for IPv6 \[[RFC8200](https://datatracker.ietf.org/doc/html/rfc8200)\]
    IPV6_ICMP = 58,
    /// No Next Header for IPv6 \[[RFC8200](https://datatracker.ietf.org/doc/html/rfc8200)\]
    IPV6_NO_NEXT_HEADER = 59,
    /// Destination Options for IPv6 \[[RFC8200](https://datatracker.ietf.org/doc/html/rfc8200)\]
    IPV6_DESTINATION_OPTIONS = 60,
    /// any host internal protocol \[Internet_Assigned_Numbers_Authority\]
    ANY_HOST_INTERNAL_PROTOCOL = 61,
    /// CFTP \[Forsdick, H., "CFTP", Network Message, Bolt Beranek and Newman, January 1982.\]\[Harry_Forsdick\]
    CFTP = 62,
    /// any local network \[Internet_Assigned_Numbers_Authority\]
    ANY_LOCAL_NETWORK = 63,
    /// SATNET and Backroom EXPAK \[Steven_Blumenthal\]
    SAT_EXPAK = 64,
    /// Kryptolan \[Paul Liu\]
    KRYTOLAN = 65,
    /// MIT Remote Virtual Disk Protocol \[Michael_Greenwald\]
    RVD = 66,
    /// Internet Pluribus Packet Core \[Steven_Blumenthal\]
    IPPC = 67,
    /// any distributed file system \[Internet_Assigned_Numbers_Authority\]
    ANY_DISTRIBUTED_FILE_SYSTEM = 68,
    /// SATNET Monitoring \[Steven_Blumenthal\]
    SAT_MON = 69,
    /// VISA Protocol \[Gene_Tsudik\]
    VISA = 70,
    /// Internet Packet Core Utility \[Steven_Blumenthal\]
    IPCV = 71,
    /// Computer Protocol Network Executive \[David Mittnacht\]
    CPNX = 72,
    /// Computer Protocol Heart Beat \[David Mittnacht\]
    CPHB = 73,
    /// Wang Span Network \[Victor Dafoulas\]
    WSN = 74,
    /// Packet Video Protocol \[Steve_Casner\]
    PVP = 75,
    /// Backroom SATNET Monitoring \[Steven_Blumenthal\]
    BR_SAT_MON = 76,
    /// SUN ND PROTOCOL-Temporary \[William_Melohn\]
    SUN_ND = 77,
    /// WIDEBAND Monitoring \[Steven_Blumenthal\]
    WB_MON = 78,
    /// WIDEBAND EXPAK \[Steven_Blumenthal\]
    WB_EXPAK = 79,
    /// ISO Internet Protocol \[Marshall_T_Rose\]
    ISO_IP = 80,
    /// VMTP \[Dave_Cheriton\]
    VMTP = 81,
    /// SECURE-VMTP \[Dave_Cheriton\]
    SECURE_VMTP = 82,
    /// VINES \[Brian Horn\]
    VINES = 83,
    /// Transaction Transport Protocol or Internet Protocol Traffic Manager \[Jim_Stevens\]
    TTP_OR_IPTM = 84,
    /// NSFNET-IGP \[Hans_Werner_Braun\]
    NSFNET_IGP = 85,
    /// Dissimilar Gateway Protocol \[M/A-COM Government Systems, "Dissimilar Gateway Protocol Specification, Draft Version", Contract no. CS901145, November 16, 1987.\]\[Mike_Little\]
    DGP = 86,
    /// TCF \[Guillermo_A_Loyola\]
    TCF = 87,
    /// EIGRP \[[RFC7868](https://datatracker.ietf.org/doc/html/rfc7868)\]
    EIGRP = 88,
    /// OSPFIGP \[[RFC1583](https://datatracker.ietf.org/doc/html/rfc1583)\]\[[RFC2328](https://datatracker.ietf.org/doc/html/rfc2328)\]\[[RFC5340](https://datatracker.ietf.org/doc/html/rfc5340)\]\[John_Moy\]
    OSPFIGP = 89,
    /// Sprite RPC Protocol \[Welch, B., "The Sprite Remote Procedure Call System", Technical Report, UCB/Computer Science Dept., 86/302, University of California at Berkeley, June 1986.\]\[Bruce Willins\]
    SPRITE_RPC = 90,
    /// Locus Address Resolution Protocol \[Brian Horn\]
    LARP = 91,
    /// Multicast Transport Protocol \[Susie_Armstrong\]
    MTP = 92,
    /// AX.25 Frames \[Brian_Kantor\]
    AX25 = 93,
    /// IP-within-IP Encapsulation Protocol \[John_Ioannidis\]
    IPIP = 94,
    /// Mobile Internetworking Control Pro. (deprecated) \[John_Ioannidis\]
    MICP = 95,
    /// Semaphore Communications Sec. Pro. \[Howard_Hart\]
    SCC_SP = 96,
    /// Ethernet-within-IP Encapsulation \[[RFC3378](https://datatracker.ietf.org/doc/html/rfc3378)\]
    ETHER_IP = 97,
    /// Encapsulation Header \[[RFC1241](https://datatracker.ietf.org/doc/html/rfc1241)\]\[Robert_Woodburn\]
    ENCAP = 98,
    /// GMTP \[\[RXB5\]\]
    GMTP = 100,
    /// Ipsilon Flow Management Protocol \[Bob_Hinden\]\[November 1995, 1997.\]
    IFMP = 101,
    /// PNNI over IP \[Ross_Callon\]
    PNNI = 102,
    /// Protocol Independent Multicast \[[RFC7761](https://datatracker.ietf.org/doc/html/rfc7761)\]\[Dino_Farinacci\]
    PIM = 103,
    /// ARIS \[Nancy_Feldman\]
    ARIS = 104,
    /// SCPS \[Robert_Durst\]
    SCPS = 105,
    /// QNX \[Michael_Hunter\]
    QNX = 106,
    /// Active Networks \[Bob_Braden\]
    ACTIVE_NETWORKS = 107,
    /// IP Payload Compression Protocol \[[RFC2393](https://datatracker.ietf.org/doc/html/rfc2393)\]
    IP_COMP = 108,
    /// Sitara Networks Protocol \[Manickam_R_Sridhar\]
    SITRA_NETWORKS_PROTOCOL = 109,
    /// Compaq Peer Protocol \[Victor_Volpe\]
    COMPAQ_PEER = 110,
    /// IPX in IP \[CJ_Lee\]
    IPX_IN_IP = 111,
    /// Virtual Router Redundancy Protocol \[[RFC5798](https://datatracker.ietf.org/doc/html/rfc5798)\]
    VRRP = 112,
    /// PGM Reliable Transport Protocol \[Tony_Speakman\]
    PGM = 113,
    /// any 0-hop protocol \[Internet_Assigned_Numbers_Authority\]
    ANY_ZERO_HOP_PROTOCOL = 114,
    /// Layer Two Tunneling Protocol \[[RFC3931](https://datatracker.ietf.org/doc/html/rfc3931)\]\[Bernard_Aboba\]
    LAYER2_TUNNELING_PROTOCOL = 115,
    /// D-II Data Exchange (DDX) \[John_Worley\]
    DDX = 116,
    /// Interactive Agent Transfer Protocol \[John_Murphy\]
    IATP = 117,
    /// Schedule Transfer Protocol \[Jean_Michel_Pittet\]
    STP = 118,
    /// SpectraLink Radio Protocol \[Mark_Hamilton\]
    SRP = 119,
    /// UTI \[Peter_Lothberg\]
    UTI = 120,
    /// Simple Message Protocol \[Leif_Ekblad\]
    SIMPLE_MESSAGE_PROTOCOL = 121,
    /// Simple Multicast Protocol (deprecated) \[Jon_Crowcroft\]\[draft-perlman-simple-multicast\]
    SM = 122,
    /// Performance Transparency Protocol \[Michael_Welzl\]
    PTP = 123,
    /// ISIS over IPv4 \[Tony_Przygienda\]
    ISIS_OVER_IPV4 = 124,
    /// FIRE \[Criag_Partridge\]
    FIRE = 125,
    /// Combat Radio Transport Protocol \[Robert_Sautter\]
    CRTP = 126,
    /// Combat Radio User Datagram \[Robert_Sautter\]
    CRUDP = 127,
    /// SSCOPMCE \[Kurt_Waber\]
    SSCOPMCE = 128,
    /// IPLT \[\[Hollbach\]\]
    IPLT = 129,
    /// Secure Packet Shield \[Bill_McIntosh\]
    SPS = 130,
    /// Private IP Encapsulation within IP \[Bernhard_Petri\]
    PIPE = 131,
    /// Stream Control Transmission Protocol \[Randall_R_Stewart\]
    SCTP = 132,
    /// Fibre Channel \[Murali_Rajagopal\]\[[RFC6172](https://datatracker.ietf.org/doc/html/rfc6172)\]
    FC = 133,
    /// RSVP-E2E-IGNORE \[[RFC3175](https://datatracker.ietf.org/doc/html/rfc3175)\]
    RSVP_E2E_IGNORE = 134,
    /// MobilityHeader \[[RFC6275](https://datatracker.ietf.org/doc/html/rfc6275)\]
    MOBILITY_HEADER = 135,
    /// UDPLite \[[RFC3828](https://datatracker.ietf.org/doc/html/rfc3828)\]
    UDP_LITE = 136,
    /// \[[RFC4023](https://datatracker.ietf.org/doc/html/rfc4023)\]
    MPLS_IN_IP = 137,
    /// MANET Protocols \[[RFC5498](https://datatracker.ietf.org/doc/html/rfc5498)\]
    MANET = 138,
    /// Host Identity Protocol \[[RFC7401](https://datatracker.ietf.org/doc/html/rfc7401)\]
    HIP = 139,
    /// Shim6 Protocol \[[RFC5533](https://datatracker.ietf.org/doc/html/rfc5533)\]
    SHIM6 = 140,
    /// Wrapped Encapsulating Security Payload \[[RFC5840](https://datatracker.ietf.org/doc/html/rfc5840)\]
    WESP = 141,
    /// Robust Header Compression \[[RFC5858](https://datatracker.ietf.org/doc/html/rfc5858)\]
    ROHC = 142,
    /// Use for experimentation and testing
    EXPERIMENTAL_AND_TESTING_0 = 253,
    /// Use for experimentation and testing
    EXPERIMENTAL_AND_TESTING_1 = 254,
    Other(u8),
}

impl From<u8> for InetProtocol {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::IPV6_HEADER_HOP_BY_HOP,
            1 => Self::ICMP,
            2 => Self::IGMP,
            3 => Self::GGP,
            4 => Self::IPV4,
            5 => Self::STREAM,
            6 => Self::TCP,
            7 => Self::CBT,
            8 => Self::EGP,
            9 => Self::IGP,
            10 => Self::BBN_RCC_MON,
            11 => Self::NVP_II,
            12 => Self::PUP,
            13 => Self::ARGUS,
            14 => Self::EMCON,
            15 => Self::XNET,
            16 => Self::CHAOS,
            17 => Self::UDP,
            18 => Self::MUX,
            19 => Self::DCN_MEAS,
            20 => Self::HMP,
            21 => Self::PRM,
            22 => Self::XNS_IDP,
            23 => Self::TRUNK1,
            24 => Self::TRUNK2,
            25 => Self::LEAF1,
            26 => Self::LEAF2,
            27 => Self::RDP,
            28 => Self::IRTP,
            29 => Self::ISO_TP4,
            30 => Self::NET_BLT,
            31 => Self::MFE_NSP,
            32 => Self::MERIT_INP,
            33 => Self::DCCP,
            34 => Self::THIRD_PARTY_CONNECT_PROTOCOL,
            35 => Self::IDPR,
            36 => Self::XTP,
            37 => Self::DDP,
            38 => Self::IDPR_CMTP,
            39 => Self::TP_PLUS_PLUS,
            40 => Self::IL,
            41 => Self::IPV6,
            42 => Self::SDRP,
            43 => Self::IPV6_ROUTE_HEADER,
            44 => Self::IPV6_FRAGMENTATION_HEADER,
            45 => Self::IDRP,
            46 => Self::RSVP,
            47 => Self::GRE,
            48 => Self::DSR,
            49 => Self::BNA,
            50 => Self::ENCAPSULATING_SECURITY_PAYLOAD,
            51 => Self::AUTHENTICATION_HEADER,
            52 => Self::INLSP,
            53 => Self::SWIPE,
            54 => Self::NARP,
            55 => Self::MOBILE,
            56 => Self::TLSP,
            57 => Self::SKIP,
            58 => Self::IPV6_ICMP,
            59 => Self::IPV6_NO_NEXT_HEADER,
            60 => Self::IPV6_DESTINATION_OPTIONS,
            61 => Self::ANY_HOST_INTERNAL_PROTOCOL,
            62 => Self::CFTP,
            63 => Self::ANY_LOCAL_NETWORK,
            64 => Self::SAT_EXPAK,
            65 => Self::KRYTOLAN,
            66 => Self::RVD,
            67 => Self::IPPC,
            68 => Self::ANY_DISTRIBUTED_FILE_SYSTEM,
            69 => Self::SAT_MON,
            70 => Self::VISA,
            71 => Self::IPCV,
            72 => Self::CPNX,
            73 => Self::CPHB,
            74 => Self::WSN,
            75 => Self::PVP,
            76 => Self::BR_SAT_MON,
            77 => Self::SUN_ND,
            78 => Self::WB_MON,
            79 => Self::WB_EXPAK,
            80 => Self::ISO_IP,
            81 => Self::VMTP,
            82 => Self::SECURE_VMTP,
            83 => Self::VINES,
            84 => Self::TTP_OR_IPTM,
            85 => Self::NSFNET_IGP,
            86 => Self::DGP,
            87 => Self::TCF,
            88 => Self::EIGRP,
            89 => Self::OSPFIGP,
            90 => Self::SPRITE_RPC,
            91 => Self::LARP,
            92 => Self::MTP,
            93 => Self::AX25,
            94 => Self::IPIP,
            95 => Self::MICP,
            96 => Self::SCC_SP,
            97 => Self::ETHER_IP,
            98 => Self::ENCAP,
            100 => Self::GMTP,
            101 => Self::IFMP,
            102 => Self::PNNI,
            103 => Self::PIM,
            104 => Self::ARIS,
            105 => Self::SCPS,
            106 => Self::QNX,
            107 => Self::ACTIVE_NETWORKS,
            108 => Self::IP_COMP,
            109 => Self::SITRA_NETWORKS_PROTOCOL,
            110 => Self::COMPAQ_PEER,
            111 => Self::IPX_IN_IP,
            112 => Self::VRRP,
            113 => Self::PGM,
            114 => Self::ANY_ZERO_HOP_PROTOCOL,
            115 => Self::LAYER2_TUNNELING_PROTOCOL,
            116 => Self::DDX,
            117 => Self::IATP,
            118 => Self::STP,
            119 => Self::SRP,
            120 => Self::UTI,
            121 => Self::SIMPLE_MESSAGE_PROTOCOL,
            122 => Self::SM,
            123 => Self::PTP,
            124 => Self::ISIS_OVER_IPV4,
            125 => Self::FIRE,
            126 => Self::CRTP,
            127 => Self::CRUDP,
            128 => Self::SSCOPMCE,
            129 => Self::IPLT,
            130 => Self::SPS,
            131 => Self::PIPE,
            132 => Self::SCTP,
            133 => Self::FC,
            134 => Self::RSVP_E2E_IGNORE,
            135 => Self::MOBILITY_HEADER,
            136 => Self::UDP_LITE,
            137 => Self::MPLS_IN_IP,
            138 => Self::MANET,
            139 => Self::HIP,
            140 => Self::SHIM6,
            141 => Self::WESP,
            142 => Self::ROHC,
            253 => Self::EXPERIMENTAL_AND_TESTING_0,
            254 => Self::EXPERIMENTAL_AND_TESTING_1,
            x => Self::Other(x),
        }
    }
}

impl From<InetProtocol> for u8 {
    fn from(value: InetProtocol) -> Self {
        match value {
            InetProtocol::IPV6_HEADER_HOP_BY_HOP => 0,
            InetProtocol::ICMP => 1,
            InetProtocol::IGMP => 2,
            InetProtocol::GGP => 3,
            InetProtocol::IPV4 => 4,
            InetProtocol::STREAM => 5,
            InetProtocol::TCP => 6,
            InetProtocol::CBT => 7,
            InetProtocol::EGP => 8,
            InetProtocol::IGP => 9,
            InetProtocol::BBN_RCC_MON => 10,
            InetProtocol::NVP_II => 11,
            InetProtocol::PUP => 12,
            InetProtocol::ARGUS => 13,
            InetProtocol::EMCON => 14,
            InetProtocol::XNET => 15,
            InetProtocol::CHAOS => 16,
            InetProtocol::UDP => 17,
            InetProtocol::MUX => 18,
            InetProtocol::DCN_MEAS => 19,
            InetProtocol::HMP => 20,
            InetProtocol::PRM => 21,
            InetProtocol::XNS_IDP => 22,
            InetProtocol::TRUNK1 => 23,
            InetProtocol::TRUNK2 => 24,
            InetProtocol::LEAF1 => 25,
            InetProtocol::LEAF2 => 26,
            InetProtocol::RDP => 27,
            InetProtocol::IRTP => 28,
            InetProtocol::ISO_TP4 => 29,
            InetProtocol::NET_BLT => 30,
            InetProtocol::MFE_NSP => 31,
            InetProtocol::MERIT_INP => 32,
            InetProtocol::DCCP => 33,
            InetProtocol::THIRD_PARTY_CONNECT_PROTOCOL => 34,
            InetProtocol::IDPR => 35,
            InetProtocol::XTP => 36,
            InetProtocol::DDP => 37,
            InetProtocol::IDPR_CMTP => 38,
            InetProtocol::TP_PLUS_PLUS => 39,
            InetProtocol::IL => 40,
            InetProtocol::IPV6 => 41,
            InetProtocol::SDRP => 42,
            InetProtocol::IPV6_ROUTE_HEADER => 43,
            InetProtocol::IPV6_FRAGMENTATION_HEADER => 44,
            InetProtocol::IDRP => 45,
            InetProtocol::RSVP => 46,
            InetProtocol::GRE => 47,
            InetProtocol::DSR => 48,
            InetProtocol::BNA => 49,
            InetProtocol::ENCAPSULATING_SECURITY_PAYLOAD => 50,
            InetProtocol::AUTHENTICATION_HEADER => 51,
            InetProtocol::INLSP => 52,
            InetProtocol::SWIPE => 53,
            InetProtocol::NARP => 54,
            InetProtocol::MOBILE => 55,
            InetProtocol::TLSP => 56,
            InetProtocol::SKIP => 57,
            InetProtocol::IPV6_ICMP => 58,
            InetProtocol::IPV6_NO_NEXT_HEADER => 59,
            InetProtocol::IPV6_DESTINATION_OPTIONS => 60,
            InetProtocol::ANY_HOST_INTERNAL_PROTOCOL => 61,
            InetProtocol::CFTP => 62,
            InetProtocol::ANY_LOCAL_NETWORK => 63,
            InetProtocol::SAT_EXPAK => 64,
            InetProtocol::KRYTOLAN => 65,
            InetProtocol::RVD => 66,
            InetProtocol::IPPC => 67,
            InetProtocol::ANY_DISTRIBUTED_FILE_SYSTEM => 68,
            InetProtocol::SAT_MON => 69,
            InetProtocol::VISA => 70,
            InetProtocol::IPCV => 71,
            InetProtocol::CPNX => 72,
            InetProtocol::CPHB => 73,
            InetProtocol::WSN => 74,
            InetProtocol::PVP => 75,
            InetProtocol::BR_SAT_MON => 76,
            InetProtocol::SUN_ND => 77,
            InetProtocol::WB_MON => 78,
            InetProtocol::WB_EXPAK => 79,
            InetProtocol::ISO_IP => 80,
            InetProtocol::VMTP => 81,
            InetProtocol::SECURE_VMTP => 82,
            InetProtocol::VINES => 83,
            InetProtocol::TTP_OR_IPTM => 84,
            InetProtocol::NSFNET_IGP => 85,
            InetProtocol::DGP => 86,
            InetProtocol::TCF => 87,
            InetProtocol::EIGRP => 88,
            InetProtocol::OSPFIGP => 89,
            InetProtocol::SPRITE_RPC => 90,
            InetProtocol::LARP => 91,
            InetProtocol::MTP => 92,
            InetProtocol::AX25 => 93,
            InetProtocol::IPIP => 94,
            InetProtocol::MICP => 95,
            InetProtocol::SCC_SP => 96,
            InetProtocol::ETHER_IP => 97,
            InetProtocol::ENCAP => 98,
            InetProtocol::GMTP => 100,
            InetProtocol::IFMP => 101,
            InetProtocol::PNNI => 102,
            InetProtocol::PIM => 103,
            InetProtocol::ARIS => 104,
            InetProtocol::SCPS => 105,
            InetProtocol::QNX => 106,
            InetProtocol::ACTIVE_NETWORKS => 107,
            InetProtocol::IP_COMP => 108,
            InetProtocol::SITRA_NETWORKS_PROTOCOL => 109,
            InetProtocol::COMPAQ_PEER => 110,
            InetProtocol::IPX_IN_IP => 111,
            InetProtocol::VRRP => 112,
            InetProtocol::PGM => 113,
            InetProtocol::ANY_ZERO_HOP_PROTOCOL => 114,
            InetProtocol::LAYER2_TUNNELING_PROTOCOL => 115,
            InetProtocol::DDX => 116,
            InetProtocol::IATP => 117,
            InetProtocol::STP => 118,
            InetProtocol::SRP => 119,
            InetProtocol::UTI => 120,
            InetProtocol::SIMPLE_MESSAGE_PROTOCOL => 121,
            InetProtocol::SM => 122,
            InetProtocol::PTP => 123,
            InetProtocol::ISIS_OVER_IPV4 => 124,
            InetProtocol::FIRE => 125,
            InetProtocol::CRTP => 126,
            InetProtocol::CRUDP => 127,
            InetProtocol::SSCOPMCE => 128,
            InetProtocol::IPLT => 129,
            InetProtocol::SPS => 130,
            InetProtocol::PIPE => 131,
            InetProtocol::SCTP => 132,
            InetProtocol::FC => 133,
            InetProtocol::RSVP_E2E_IGNORE => 134,
            InetProtocol::MOBILITY_HEADER => 135,
            InetProtocol::UDP_LITE => 136,
            InetProtocol::MPLS_IN_IP => 137,
            InetProtocol::MANET => 138,
            InetProtocol::HIP => 139,
            InetProtocol::SHIM6 => 140,
            InetProtocol::WESP => 141,
            InetProtocol::ROHC => 142,
            InetProtocol::EXPERIMENTAL_AND_TESTING_0 => 253,
            InetProtocol::EXPERIMENTAL_AND_TESTING_1 => 254,
            InetProtocol::Other(x) => x,
        }
    }
}
