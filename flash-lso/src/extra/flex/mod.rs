const NEXT_FLAG: u8 = 128;

const BODY_FLAG: u8 = 1;
const CLIENT_ID_FLAG: u8 = 2;
const DESTINATION_ID_FLAG: u8 = 4;
const HEADERS_FLAG: u8 = 8;
const MESSAGE_ID_FLAG: u8 = 16;
const TIMESTAMP_FLAG: u8 = 32;
const TTL_FLAG: u8 = 64;

const CLIENT_ID_BYTES_FLAG: u8 = 1;
const MESSAGE_ID_BYTES_FLAG: u8 = 2;

const CORRELATION_ID_FLAG: u8 = 1;
const CORRELATION_ID_BYTES_FLAG: u8 = 2;

const OPERATION_FLAG: u8 = 1;

pub mod read;
pub mod write;
