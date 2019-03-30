use crate::Void;
use lv2_raw::LV2Feature;
use lv2_raw::LV2Handle;
use std::ffi::CStr;
use std::ptr;

#[repr(C)]
pub struct LV2Descriptor {
    pub uri: *const libc::c_char,
    pub instantiate: extern "C" fn(
        descriptor: *const LV2Descriptor,
        rate: f64,
        bundle_path: *const libc::c_char,
        features: *const (*const LV2Feature),
    ) -> LV2Handle,
    pub connect_port: extern "C" fn(handle: LV2Handle, port: u32, data: *mut libc::c_void),
    pub activate: Option<extern "C" fn(instance: LV2Handle)>,
    pub run: extern "C" fn(instance: LV2Handle, n_samples: u32),
    pub deactivate: Option<extern "C" fn(instance: LV2Handle)>,
    pub cleanup: extern "C" fn(instance: LV2Handle),
    pub extension_data: Option<extern "C" fn(uri: *const libc::c_char) -> (*const libc::c_void)>,
}

#[repr(C)]
pub(crate) struct InstanceImpl {
    pub(crate) descriptor: *const LV2Descriptor,
    pub(crate) handle: *mut Void,
    pub(crate) private: *mut Void,
}

impl InstanceImpl {
    #[inline(always)]
    pub unsafe fn get_uri(&self) -> *const libc::c_char {
        (*self.descriptor).uri
    }

    #[inline(always)]
    pub unsafe fn connect_port(&mut self, port_index: u32, data_location: *mut libc::c_void) {
        ((*self.descriptor).connect_port)(self.handle, port_index, data_location);
    }

    #[inline(always)]
    pub unsafe fn activate(&mut self) {
        if let Some(activate) = (*self.descriptor).activate {
            activate(self.handle)
        }
    }

    #[inline(always)]
    pub unsafe fn run(&mut self, sample_count: u32) {
        ((*self.descriptor).run)(self.handle, sample_count);
    }

    #[inline(always)]
    pub unsafe fn deactivate(&mut self) {
        if let Some(deactivate) = (*self.descriptor).deactivate {
            deactivate(self.handle)
        }
    }

    #[inline(always)]
    pub unsafe fn get_descriptor(&self) -> *const LV2Descriptor {
        self.descriptor
    }
}

#[repr(C)]
pub struct Instance(pub(crate) *mut InstanceImpl);

unsafe impl Send for Instance {}

impl Instance {
    #[inline(always)]
    pub unsafe fn get_uri(&self) -> &CStr {
        CStr::from_ptr((*self.0).get_uri())
    }

    #[inline(always)]
    pub unsafe fn connect_port(&mut self, port_index: u32, data_location: *mut libc::c_void) {
        (*self.0).connect_port(port_index, data_location);
    }

    #[inline(always)]
    pub unsafe fn activate(&mut self) {
        (*self.0).activate();
    }

    #[inline(always)]
    pub unsafe fn run(&mut self, sample_count: u32) {
        (*self.0).run(sample_count);
    }

    #[inline(always)]
    pub unsafe fn deactivate(&mut self) {
        (*self.0).deactivate();
    }

    #[inline(always)]
    pub unsafe fn get_extension_data(&self, uri: &CStr) -> *const libc::c_void {
        self.get_descriptor().extension_data.map_or(ptr::null_mut(), |f| f(uri.as_ptr()))
    }

    #[inline(always)]
    pub unsafe fn get_descriptor(&self) -> &LV2Descriptor {
        &*(*self.0).get_descriptor()
    }

    #[inline(always)]
    pub unsafe fn get_handle(&self) -> LV2Handle {
        (*self.0).handle
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.0;
            ((*inner.descriptor).cleanup)(inner.handle)
        };
    }
}
