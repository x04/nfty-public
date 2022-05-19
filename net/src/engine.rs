use cronet_sys::*;

#[path = "engine_builder.rs"]
mod builder;
pub use self::builder::EngineParams;

pub struct CronetEngine {
    pub(crate) engine: Cronet_EnginePtr,
}
unsafe impl Send for CronetEngine {}
unsafe impl Sync for CronetEngine {}

impl CronetEngine {
    pub fn new(params: &mut EngineParams) -> Self {
        let engine = CronetEngine {
            engine: unsafe { Cronet_Engine_Create() },
        };

        unsafe {
            Cronet_Engine_StartWithParams(engine.engine, params.params);
        }

        engine
    }
}

impl Drop for CronetEngine {
    fn drop(&mut self) {
        unsafe {
            Cronet_Engine_Shutdown(self.engine);
            Cronet_Engine_Destroy(self.engine);
        }
    }
}
