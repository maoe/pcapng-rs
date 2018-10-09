use block::RawBlock;
use nom::{le_u32, IResult};
use options::{parse_options, Options};
use util;

pub const TY: u32 = 0x00000006;

//    0                   1                   2                   3
//    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//    +---------------------------------------------------------------+
//  0 |                    Block Type = 0x00000006                    |
//    +---------------------------------------------------------------+
//  4 |                      Block Total Length                       |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//  8 |                         Interface ID                          |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 12 |                        Timestamp (High)                       |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 16 |                        Timestamp (Low)                        |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 20 |                         Captured Len                          |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 24 |                          Packet Len                           |
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
// 28 /                                                               /
//    /                          Packet Data                          /
//    /          /* variable length, aligned to 32 bits */            /
//    /                                                               /
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    /                                                               /
//    /                      Options (variable)                       /
//    /                                                               /
//    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//    |                      Block Total Length                       |
//    +---------------------------------------------------------------+

named!(enhanced_packet_body<&[u8],EnhancedPacket>,
       do_parse!(
              interface_id: le_u32
           >> timestamp_hi: le_u32
           >> timestamp_lo: le_u32
           >> captured_len: le_u32
           >> packet_len:   le_u32

           // Captured Len: number of bytes captured from the packet (i.e. the length of the Packet
           // Data field). It will be the minimum value among the actual Packet Length and the
           // snapshot length (defined in Figure 9). The value of this field does not include the
           // padding bytes added at the end of the Packet Data field to align the Packet Data

           // Field to a 32-bit boundary
           >> data: take!(captured_len as usize)
           >> take!(util::pad_to_32bits(captured_len as usize))
           >> options: opt!(complete!(parse_options))


           >> (
               EnhancedPacket {
                   ty: TY,
                   block_length: 0,
                   interface_id: interface_id,
                   timestamp_hi: timestamp_hi,
                   timestamp_lo: timestamp_lo,
                   captured_len: captured_len,
                   packet_len: packet_len,
                   data: data,
                   options: options,
                   check_length: 0,
               }
           )
       )
);

pub fn parse(blk: RawBlock) -> IResult<&[u8], EnhancedPacket> {
    match enhanced_packet_body(blk.body) {
        // FIXME(richo) actually do something with the leftover bytes
        Ok((left, mut block)) => {
            block.block_length = blk.block_length;
            block.check_length = blk.check_length;
            Ok((left, block))
        }
        Err(e) => Err(e),
    }
}

#[derive(Debug)]
/// An Enhanced Packet Block is the standard container for storing the packets coming from the
/// network.
pub struct EnhancedPacket<'a> {
    pub ty: u32,
    pub block_length: u32,
    pub interface_id: u32,
    pub timestamp_hi: u32,
    pub timestamp_lo: u32,
    pub captured_len: u32,
    pub packet_len: u32,
    pub data: &'a [u8],
    pub options: Option<Options<'a>>,
    pub check_length: u32,
}
