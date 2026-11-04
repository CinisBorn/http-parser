/// Max request line size for a request. By default the size is `8kib`. 
pub const MAX_REQUEST_LINE_SIZE: usize = 8 * 1024;
/// Default header max size. 
pub const MAX_HEADER_SIZE: usize = 16 * 1024;
/// Default number of headers.
pub const MAX_HEADER_COUNT: usize = 100;
/// Default max body size value.
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;