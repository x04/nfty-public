use cronet_sys::*;
use std::ffi::CString;

pub struct EngineParams {
    pub(super) params: Cronet_EngineParamsPtr,
}
unsafe impl Send for EngineParams {}
unsafe impl Sync for EngineParams {}

impl EngineParams {
    pub fn new() -> Self {
        Self {
            params: unsafe { Cronet_EngineParams_Create() },
        }
    }

    pub fn set_user_agent(&mut self, user_agent: &str) {
        unsafe {
            let c_string = CString::new(user_agent.as_bytes()).expect("Null byte in user agent");
            Cronet_EngineParams_user_agent_set(self.params, c_string.as_c_str().as_ptr());
        }
    }

    pub fn set_accept_language(&mut self, accept_language: &str) {
        unsafe {
            let c_string =
                CString::new(accept_language.as_bytes()).expect("Null byte in accept language");
            Cronet_EngineParams_accept_language_set(self.params, c_string.as_c_str().as_ptr());
        }
    }

    pub fn set_experimental_options(&mut self, experimental_options: &str) {
        unsafe {
            let c_string = CString::new(experimental_options.as_bytes())
                .expect("Null byte in experimental options");
            Cronet_EngineParams_experimental_options_set(self.params, c_string.as_c_str().as_ptr());
        }
    }

    pub fn set_proxy_uri(&mut self, proxy: &str) {
        unsafe {
            let c_string = CString::new(proxy.as_bytes()).expect("Null byte in proxy");
            Cronet_EngineParams_proxy_uri_set(self.params, c_string.as_c_str().as_ptr());
        }
    }

    pub fn set_proxy_credentials(&mut self, credentials: &str) {
        unsafe {
            let c_string = CString::new(credentials.as_bytes()).expect("Null byte in proxy");
            Cronet_EngineParams_proxy_credentials_set(self.params, c_string.as_c_str().as_ptr());
        }
    }

    pub fn set_brotli(&mut self, brotli: bool) {
        unsafe {
            Cronet_EngineParams_enable_brotli_set(self.params, brotli);
        }
    }

    pub fn set_http2(&mut self, http2: bool) {
        unsafe {
            Cronet_EngineParams_enable_http2_set(self.params, http2);
        }
    }

    pub fn set_quic(&mut self, quic: bool) {
        unsafe {
            Cronet_EngineParams_enable_quic_set(self.params, quic);
        }
    }
}

impl Default for EngineParams {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EngineParams {
    fn drop(&mut self) {
        unsafe {
            Cronet_EngineParams_Destroy(self.params);
        }
    }
}
