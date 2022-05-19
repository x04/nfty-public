use crate::request::callback::CallbackData;
use cronet_sys::*;
use std::ffi::c_void;

// A wrapper around CallbackData that owns the allocation.
pub(crate) struct CallbackAlloc {
    data: *mut CallbackData,
    pub(super) callback_ptr: Cronet_UrlRequestCallbackPtr,
}

impl CallbackAlloc {
    pub fn new(data: CallbackData) -> Self {
        unsafe {
            let data = Box::into_raw(Box::new(data));

            let callback_ptr = Cronet_UrlRequestCallback_CreateWith(
                Some(on_redirect),
                Some(on_response_start),
                Some(on_read_completed),
                Some(on_succeeded),
                Some(on_failed),
                Some(on_canceled),
            );

            Cronet_UrlRequestCallback_SetClientContext(callback_ptr, data as *mut c_void);

            Self { data, callback_ptr }
        }
    }
}

impl Drop for CallbackAlloc {
    fn drop(&mut self) {
        unsafe {
            Cronet_UrlRequestCallback_Destroy(self.callback_ptr);
            drop(Box::from_raw(self.data));
        }
    }
}

unsafe extern "C" fn on_redirect(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
    new_location_url: Cronet_String,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_redirect(request, info, new_location_url);
}

unsafe extern "C" fn on_response_start(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_response_start(request, info);
}

unsafe extern "C" fn on_read_completed(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
    buffer: Cronet_BufferPtr,
    bytes_read: u64,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_read_completed(request, info, buffer, bytes_read);
}

unsafe extern "C" fn on_succeeded(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_succeeded(request, info);
}

unsafe extern "C" fn on_failed(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
    error: Cronet_ErrorPtr,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_failed(request, info, error);
}

unsafe extern "C" fn on_canceled(
    _self: Cronet_UrlRequestCallbackPtr,
    request: Cronet_UrlRequestPtr,
    info: Cronet_UrlResponseInfoPtr,
) {
    let data = Cronet_UrlRequestCallback_GetClientContext(_self) as *mut CallbackData;
    (*data).on_canceled(request, info);
}
