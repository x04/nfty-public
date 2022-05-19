use super::{callback::CallbackData, callback_alloc::CallbackAlloc, CronetRequest, Response};
use cronet_sys::*;
use futures::channel::oneshot;
use indexmap::IndexMap;
use std::{
    ffi::CString,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// A running cronet request. This type implements `Future`, which means you can
/// `.await` it in async code, producing a [`Response`].
///
/// [`Response`]: struct.Response.html
pub struct RunningRequest<'a> {
    request: Cronet_UrlRequestPtr,
    recv: oneshot::Receiver<Response>,

    // included only for destructor
    #[allow(dead_code)]
    callback: CallbackAlloc,

    lifetime: PhantomData<&'a ()>,
}
unsafe impl<'a> Send for RunningRequest<'a> {}
unsafe impl<'a> Sync for RunningRequest<'a> {}
impl<'a> Unpin for RunningRequest<'a> {}

impl<'a> RunningRequest<'a> {
    pub(super) fn new(req: &mut CronetRequest<'a>, url: &str) -> Self {
        unsafe {
            let (data, recv) = CallbackData::new();
            let callback = CallbackAlloc::new(data);

            let url = CString::new(url.as_bytes()).expect("Null byte in url");

            let request = Cronet_UrlRequest_Create();
            Cronet_UrlRequest_InitWithParams(
                request,
                req.engine,
                url.as_c_str().as_ptr(),
                req.request_params,
                callback.callback_ptr,
                req.executor,
            );

            Cronet_UrlRequest_Start(request);

            Self {
                request,
                callback,
                recv,
                lifetime: PhantomData,
            }
        }
    }
}

impl<'a> Future for RunningRequest<'a> {
    type Output = Response;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Response> {
        let me = Pin::into_inner(self);
        match Pin::new(&mut me.recv).poll(cx) {
            Poll::Ready(Ok(response)) => Poll::Ready(response),
            Poll::Ready(Err(oneshot::Canceled)) => Poll::Ready(Response {
                status_code: -1,
                status: super::Status::Failed,
                last_error: Some("request killed without telling why".to_string()),
                headers: Some(IndexMap::new()),
                body: Vec::new(),
            }),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a> Drop for RunningRequest<'a> {
    fn drop(&mut self) {
        unsafe {
            Cronet_UrlRequest_Destroy(self.request);
        }
    }
}
