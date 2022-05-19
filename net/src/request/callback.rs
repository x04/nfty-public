use super::{Response, Status};
use cronet_sys::*;
use futures::channel::oneshot;
use indexmap::IndexMap;
use std::{ffi::CStr, mem};

pub(crate) struct CallbackData {
    last_error: Option<String>,
    status_code: Option<i32>,
    headers: Option<IndexMap<String, Vec<String>>>,
    response_data: Vec<u8>,
    chan: Option<oneshot::Sender<Response>>,
}

impl CallbackData {
    pub fn new() -> (Self, oneshot::Receiver<Response>) {
        let (send, recv) = oneshot::channel();
        (
            Self {
                last_error: None,
                response_data: Vec::new(),
                status_code: None,
                headers: None,
                chan: Some(send),
            },
            recv,
        )
    }

    pub unsafe fn on_redirect(
        &mut self,
        request: Cronet_UrlRequestPtr,
        _info: Cronet_UrlResponseInfoPtr,
        _new_location_url: Cronet_String,
    ) {
        Cronet_UrlRequest_FollowRedirect(request);
    }

    pub unsafe fn on_response_start(
        &mut self,
        request: Cronet_UrlRequestPtr,
        info: Cronet_UrlResponseInfoPtr,
    ) {
        self.status_code = Some(Cronet_UrlResponseInfo_http_status_code_get(info));

        let mut headers = IndexMap::<String, Vec<String>>::new();

        for i in 0..Cronet_UrlResponseInfo_all_headers_list_size(info) {
            let header = Cronet_UrlResponseInfo_all_headers_list_at(info, i);
            let h_key = CStr::from_ptr(Cronet_HttpHeader_name_get(header))
                .to_str()
                .unwrap()
                .to_owned();
            let h_value = CStr::from_ptr(Cronet_HttpHeader_value_get(header))
                .to_str()
                .unwrap()
                .to_owned();

            headers.entry(h_key).or_default().push(h_value);
        }

        self.headers = Some(headers);

        // Start reading response.
        let buffer = Cronet_Buffer_Create();
        Cronet_Buffer_InitWithAlloc(buffer, 32 * 1024);
        Cronet_UrlRequest_Read(request, buffer);
    }

    pub unsafe fn on_read_completed(
        &mut self,
        request: Cronet_UrlRequestPtr,
        _info: Cronet_UrlResponseInfoPtr,
        buffer: Cronet_BufferPtr,
        bytes_read: u64,
    ) {
        let data = std::slice::from_raw_parts(
            Cronet_Buffer_GetData(buffer) as *const u8,
            bytes_read as usize,
        );

        self.response_data.extend(data);

        // Continue reading the response.
        Cronet_UrlRequest_Read(request, buffer);
    }

    fn send_response(&mut self, status: Status) {
        let resp = Response {
            status_code: self.status_code.unwrap_or(-1),
            status,
            last_error: mem::take(&mut self.last_error),
            headers: mem::take(&mut self.headers),
            body: mem::take(&mut self.response_data),
        };

        if let Some(chan) = self.chan.take() {
            let _ = chan.send(resp);
        }
    }

    pub unsafe fn on_succeeded(
        &mut self,
        _request: Cronet_UrlRequestPtr,
        _info: Cronet_UrlResponseInfoPtr,
    ) {
        self.send_response(Status::Succeeded);
    }

    #[allow(unused_variables)]
    pub unsafe fn on_failed(
        &mut self,
        _request: Cronet_UrlRequestPtr,
        _info: Cronet_UrlResponseInfoPtr,
        error: Cronet_ErrorPtr,
    ) {
        let last_error = CStr::from_ptr(Cronet_Error_message_get(error));
        self.last_error = Some(last_error.to_string_lossy().into_owned());
        self.send_response(Status::Failed);
    }

    pub unsafe fn on_canceled(
        &mut self,
        _request: Cronet_UrlRequestPtr,
        _info: Cronet_UrlResponseInfoPtr,
    ) {
        self.send_response(Status::Canceled);
    }
}
