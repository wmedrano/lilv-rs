use lilv_sys::*;
use lv2_raw::LV2Descriptor;
use lv2_raw::LV2Handle;
use std::ffi::CStr;

#[repr(C)]
pub(crate) struct InstanceImpl(LilvInstance);

impl InstanceImpl {
    #[inline(always)]
    pub unsafe fn get_uri(&self) -> *const libc::c_char {
        (*self.0.lv2_descriptor).uri
    }

    #[inline(always)]
    pub unsafe fn connect_port(&mut self, port_index: u32, data_location: *mut libc::c_void) {
        ((*self.0.lv2_descriptor).connect_port)(self.0.lv2_handle, port_index, data_location);
    }

    #[inline(always)]
    pub unsafe fn activate(&mut self) {
        if let Some(activate) = (*self.0.lv2_descriptor).activate {
            activate(self.0.lv2_handle)
        }
    }

    #[inline(always)]
    pub unsafe fn run(&mut self, sample_count: u32) {
        ((*self.0.lv2_descriptor).run)(self.0.lv2_handle, sample_count);
    }

    #[inline(always)]
    pub unsafe fn deactivate(&mut self) {
        if let Some(deactivate) = (*self.0.lv2_descriptor).deactivate {
            deactivate(self.0.lv2_handle)
        }
    }

    #[inline(always)]
    pub unsafe fn get_descriptor(&self) -> *const LV2Descriptor {
        self.0.lv2_descriptor
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
        let f = self.get_descriptor().extension_data;
        f(uri.as_ptr() as *const u8)
    }

    #[inline(always)]
    pub unsafe fn get_descriptor(&self) -> &LV2Descriptor {
        &*(*self.0).get_descriptor()
    }

    #[inline(always)]
    pub unsafe fn get_handle(&self) -> LV2Handle {
        (*self.0).0.lv2_handle
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.0;
            ((*inner.0.lv2_descriptor).cleanup)(inner.0.lv2_handle)
        };
    }
}
