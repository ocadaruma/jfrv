// use std::fs::File;
// use std::io::{Read, Seek};
//
// const CHUNK_HEADER_SIZE: usize = 68;
//
// enum Error {
//     InvalidFormat,
// }
//
// type Result<T> = std::result::Result<T, Error>;
//
// struct Event;
//
// enum SampleType {
//     ExecutionSample,
//     AllocationSample,
//     ContendedLock,
// }
//
// struct JfrReader<R> where R: Read + Seek {
//     reader: R,
//     len: usize,
// }
//
// impl<R> JfrReader<R> where R: Read + Seek {
//     pub fn read(&mut self) -> Result<Event> {
//         let pos = self.reader.stream_position()?;
//         let remaining = self.len - pos;
//     }
//
//     fn read_varint(&mut self) -> Result<i32> {
//         let mut result: i32 = 0;
//         let mut shift: i32 = 0;
//         loop {
//             self.buf_reader.read_u8()
//             let b = self.buf_reader.read_be()?;
//             result |= (bi32 & 0x7f) << shift;
//             if b >= 0 {
//                 return Ok(result);
//             }
//             shift += 7;
//         }
//     }
//
//     // fn read_varlong(&mut self) -> Result<i64, ReaderError> {
//     //     let mut result: i64 = 0;
//     //     let mut shift: i32 = 0;
//     //     loop {
//     //         let b = self.buf_reader.read_be()?;
//     //         result |= (bi64 & 0x7f) << shift;
//     //         if b >= 0 {
//     //             return Ok(result);
//     //         }
//     //         shift += 7;
//     //     }
//     // }
// }
//
// struct ChunkHeader {
//     major: i16,
//     minor: i16,
//     chunk_size: i64,
//     constant_pool_offset: i64,
//     metadata_offset: i64,
//     start_time_nanos: i64,
//     duration_nanos: i64,
//     start_ticks: i64,
//     ticks_per_second: i64,
//     features: i32,
// }
//
// struct ChunkReader {
//
// }
//
// impl ChunkReader {
//     pub fn read_header(&mut self) -> Result<ChunkHeader> {
//         let pos = self.reader.stream_position()?;
//         let remaining = self.len - pos;
//     }
// }
