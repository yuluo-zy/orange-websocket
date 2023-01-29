use std::io::Write;
use crate::protocol::header::{DataFrameFlags, DataFrameHeader, DataMasker, FrameHeader, gen_mask};
use crate::result::WebSocketResult;

// 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
// +-+-+-+-+-------+-+-------------+-------------------------------+
// |F|R|R|R| opcode|M| Payload len |    Extended payload length    |
// |I|S|S|S|  (4)  |A|     (7)     |             (16/64)           |
// |N|V|V|V|       |S|             |   (if payload len==126/127)   |
// | |1|2|3|       |K|             |                               |
// +-+-+-+-+-------+-+-------------+ - - - - - - - - - - - - - - - +
// |     Extended payload length continued, if payload len == 127  |
// + - - - - - - - - - - - - - - - +-------------------------------+
// |                               |Masking-key, if MASK set to 1  |
// +-------------------------------+-------------------------------+
// | Masking-key (continued)       |          Payload Data         |
// +-------------------------------- - - - - - - - - - - - - - - - +
// :                     Payload Data continued ...                :
// + - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - +
// |                     Payload Data continued ...                |
// +---------------------------------------------------------------+
// Mask: 1 bit
// ​	mask标志位，定义“有效负载数据”是否添加掩码。如果设置为1，那么掩码的键值存在于Masking-Key中，根据5.3节描述，这个一般用于解码“有效负载数据”。所有的从客户端发送到服务端的帧都需要设置这个bit位为1。
/// Masking-Key: 0 or 4 bytes
// ​	所有从客户端发往服务端的数据帧都已经与一个包含在这一帧中的32 bit的掩码进行过了运算。如果mask标志位（1 bit）为1，那么这个字段存在，如果标志位为0，那么这个字段不存在。在5.3节中会介绍更多关于客户端到服务端增加掩码的信息。
// Payload data: (x+y) bytes
// ​	“有效负载数据”是指“扩展数据”和“应用数据”。
// Extension data: x bytes
// ​	除非协商过扩展，否则“扩展数据”长度为0 bytes。在握手协议中，任何扩展都必须指定“扩展数据”的长度，这个长度如何进行计算，以及这个扩展如何使用。如果存在扩展，那么这个“扩展数据”包含在总的有效负载长度中。
// Application data: y bytes
// ​	任意的“应用数据”，占用“扩展数据”后面的剩余所有字段。“应用数据”的长度等于有效负载长度减去“扩展应用”长度。
// 基础数据帧协议通过ABNF进行了正式的定义。需要重点知道的是，这些数据都是二进制的，而不是ASCII字符。例如，长度为1 bit的字段的值为%x0 / %x1代表的是一个值为0/1的单独的bit，而不是一整个字节（8 bit）来代表ASCII编码的字符“0”和“1”。一个长度为4 bit的范围是%x0-F的字段值代表的是4个bit，而不是字节（8 bit）对应的ASCII码的值。不要指定字符编码：“规则解析为一组最终的值，有时候是字符。在ABNF中，字符仅仅是一个非负的数字。在特定的上下文中，会根据特定的值的映射（编码）编码集（例如ASCII）”。在这里，指定的编码类型是将每个字段编码为特定的bits数组的二进制编码的最终数据。

pub trait DataFrame {
    /// FIN: 1 bit 表示这是消息的最后一个片段。第一个片段也有可能是最后一个片段。
    fn is_last(&self) -> bool;

    /// Opcode:  4 bit 定义“有效负载数据”的解释。如果收到一个未知的操作码，接收终端必须断开WebSocket连接。下面的值是被定义过的。
    //  %x0 表示一个持续帧
    // 	%x1 表示一个文本帧
    // 	%x2 表示一个二进制帧
    // 	%x3-7 预留给以后的非控制帧
    // 	%x8 表示一个连接关闭包
    // 	%x9 表示一个ping包
    // 	%xA 表示一个pong包
    // 	%xB-F 预留给以后的控制帧
    fn opcode(&self) -> u8;

    /// RSV1，RSV2，RSV3: 每个1 bit 必须设置为0，除非扩展了非0值含义的扩展。如果收到了一个非0值但是没有扩展任何非0值的含义，接收终端必须断开WebSocket连接。
    fn reserved(&self) -> &[bool; 3];

    /// Payload length: 7 bits, 7+16 bits, or 7+64 bits
    /// 以字节为单位的“有效负载数据”长度，如果值为0-125，那么就表示负载数据的长度。
    /// 如果是126，那么接下来的2个bytes解释为16bit的无符号整形作为负载数据的长度。
    /// 如果是127，那么接下来的8个bytes解释为一个64bit的无符号整形（最高位的bit必须为0）作为负载数据的长度。
    /// 多字节长度量以网络字节顺序表示（译注：应该是指大端序和小端序）
    fn size(&self) -> usize;

    /// 完整的数据帧大小 以字节为单位,
    fn frame_size(&self, masked: bool) -> usize {
        // one byte for the opcode & reserved & fin
        1
            // depending on the size of the payload, add the right payload len bytes
            + match self.size() {
            s if s <= 125 => 1,
            s if s <= 65535 => 3,
            _ => 9,
        }
            // add the mask size if there is one
            + if masked {
            4
        } else {
            0
        }
            // finally add the payload len
            + self.size()
    }

    /// Write the payload to a writer
    fn write_payload(&self, socket: &mut impl Write) -> WebSocketResult<()>;

    /// 获得传输数据
    fn take_payload(self) -> Vec<u8>;

    /// Writes a DataFrame to a Writer.
    fn write_to(&self, writer: &mut impl Write, mask: bool) -> WebSocketResult<()> {
        let mut flags = DataFrameFlags::empty();
        if self.is_last() {
            flags.insert(DataFrameFlags::FIN);
        }

        let reserved = self.reserved();
        if reserved[0] {
            flags.insert(DataFrameFlags::RSV1);
        }
        if reserved[1] {
            flags.insert(DataFrameFlags::RSV2);
        }
        if reserved[2] {
            flags.insert(DataFrameFlags::RSV3);
        }


        let masking_key = if mask { Some(gen_mask()) } else { None };

        let header = DataFrameHeader {
            flags,
            opcode: self.opcode() as u8,
            mask: masking_key,
            len: self.size() as u64,
        };

        let mut data = Vec::<u8>::new();
        header.write(&mut data)?;

        match masking_key {
            Some(mask) => {
                let mut masker = DataMasker::new(mask,&mut data);
                self.write_payload(&mut masker)?
            }
            None => self.write_payload(&mut data)?,
        };
        writer.write_all(data.as_slice())?;
        Ok(())
    }
}