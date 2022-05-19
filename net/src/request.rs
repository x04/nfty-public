use crate::{engine::CronetEngine, executor::Executor};
use cronet_sys::*;
use indexmap::IndexMap;
use std::{ffi::CString, marker::PhantomData};

mod callback;
mod callback_alloc;
mod future;
mod upload_data;

pub use self::{future::RunningRequest, upload_data::UploadData};

#[derive(Copy, Clone, Debug)]
pub enum Status {
    Succeeded,
    Failed,
    Canceled,
}

#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: i32,
    pub headers: Option<IndexMap<String, Vec<String>>>,
    pub status: Status,
    pub last_error: Option<String>,
    pub body: Vec<u8>,
}

pub struct CronetRequest<'a> {
    engine: Cronet_EnginePtr,
    executor: Cronet_ExecutorPtr,
    request_params: Cronet_UrlRequestParamsPtr,

    // This lifetime ensures that engine and executor remain valid.
    lifetime: PhantomData<&'a ()>,
}
unsafe impl<'a> Send for CronetRequest<'a> {}
unsafe impl<'a> Sync for CronetRequest<'a> {}

impl<'a> CronetRequest<'a> {
    pub fn new(engine: &'a CronetEngine, exec: &'a Executor) -> CronetRequest<'a> {
        let request_params = unsafe { Cronet_UrlRequestParams_Create() };

        Self {
            engine: engine.engine,
            executor: exec.exec_ptr,
            request_params,
            lifetime: PhantomData,
        }
    }

    /// Set the HTTP method. Should be a string like `GET`.
    pub fn set_method(&mut self, method: &str) {
        unsafe {
            let c_string = CString::new(method.as_bytes()).expect("Null byte in method");
            Cronet_UrlRequestParams_http_method_set(
                self.request_params,
                c_string.as_c_str().as_ptr(),
            );
        }
    }

    /// Set a HTTP Header
    pub fn set_header(&mut self, name: &str, value: &str) {
        unsafe {
            let c_name = CString::new(name.as_bytes()).expect("Null byte in method");
            let c_value = CString::new(value.as_bytes()).expect("Null byte in method");
            let cronet_header = Cronet_HttpHeader_Create();
            Cronet_HttpHeader_name_set(cronet_header, c_name.as_c_str().as_ptr());
            Cronet_HttpHeader_value_set(cronet_header, c_value.as_c_str().as_ptr());
            Cronet_UrlRequestParams_request_headers_add(self.request_params, cronet_header);
            Cronet_HttpHeader_Destroy(cronet_header);
        }
    }

    pub fn set_body(&mut self, body: &'a mut UploadData) {
        unsafe {
            Cronet_UrlRequestParams_upload_data_provider_set(self.request_params, body.ptr);

            Cronet_UrlRequestParams_upload_data_provider_executor_set(
                self.request_params,
                self.executor,
            );
        }
    }

    /// Start this request.
    ///
    /// The lifetime ensures that the running request doesn't outlive the engine
    /// and executor, but the CronetRequest object may be destroyed while it is running.
    pub fn start(&mut self, url: &str) -> RunningRequest<'a> {
        RunningRequest::new(self, url)
    }
}

impl<'a> Drop for CronetRequest<'a> {
    fn drop(&mut self) {
        unsafe {
            Cronet_UrlRequestParams_Destroy(self.request_params);
        }
    }
}
