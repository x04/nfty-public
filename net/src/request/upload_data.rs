use cronet_sys::*;
use std::{
    ffi::c_void,
    io::{BufRead, Cursor},
};

pub struct UploadData {
    body: *mut Cursor<Vec<u8>>,
    pub(super) ptr: Cronet_UploadDataProviderPtr,
}
unsafe impl Send for UploadData {}
unsafe impl Sync for UploadData {}

impl UploadData {
    pub fn new(body: Vec<u8>) -> Self {
        unsafe {
            let body: *mut Cursor<Vec<u8>> = Box::into_raw(Box::new(Cursor::new(body)));

            let ptr = Cronet_UploadDataProvider_CreateWith(
                Some(get_length),
                Some(read_data),
                Some(rewind),
                Some(close),
            );

            Cronet_UploadDataProvider_SetClientContext(ptr, body as *mut c_void);

            Self { body, ptr }
        }
    }
}

impl Drop for UploadData {
    fn drop(&mut self) {
        unsafe {
            Cronet_UploadDataProvider_Destroy(self.ptr);
            drop(Box::from_raw(self.body));
        }
    }
}

unsafe extern "C" fn get_length(self_: Cronet_UploadDataProviderPtr) -> i64 {
    let body = Cronet_UploadDataProvider_GetClientContext(self_);
    let body = &*(body as *const Cursor<Vec<u8>>);
    let full_len = body.get_ref().len() as i64;
    let pos = body.position() as i64;
    full_len - pos
}

unsafe extern "C" fn read_data(
    self_: Cronet_UploadDataProviderPtr,
    upload_data_sink: Cronet_UploadDataSinkPtr,
    buffer: Cronet_BufferPtr,
) {
    let body = Cronet_UploadDataProvider_GetClientContext(self_);
    let body = &mut *(body as *mut Cursor<Vec<u8>>);

    let slice = body.fill_buf().unwrap(); // guaranteed not to fail
    let len = std::cmp::min(slice.len(), Cronet_Buffer_GetSize(buffer) as usize);
    let buffer_slice =
        std::slice::from_raw_parts_mut(Cronet_Buffer_GetData(buffer) as *mut u8, len);
    buffer_slice.copy_from_slice(&slice[..len]);
    body.consume(len);

    Cronet_UploadDataSink_OnReadSucceeded(upload_data_sink, len as u64, false);
}

unsafe extern "C" fn rewind(
    self_: Cronet_UploadDataProviderPtr,
    upload_data_sink: Cronet_UploadDataSinkPtr,
) {
    let body = Cronet_UploadDataProvider_GetClientContext(self_);
    let body = &mut *(body as *mut Cursor<Vec<u8>>);

    body.set_position(0);

    Cronet_UploadDataSink_OnRewindSucceeded(upload_data_sink);
}

unsafe extern "C" fn close(_self: Cronet_UploadDataProviderPtr) {}
