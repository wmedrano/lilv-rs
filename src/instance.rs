use crate::Void;

#[repr(C)]
pub(crate) struct InstanceImpl {
    pub(crate) descriptor: *const lv2_raw::LV2Descriptor,
    pub(crate) handle: *mut Void,
    pub(crate) private: *mut Void,
}

impl InstanceImpl {
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
}

#[repr(C)]
pub struct Instance(pub(crate) *mut InstanceImpl);

unsafe impl Send for Instance {}

impl Instance {
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
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.0;
            ((*inner.descriptor).cleanup)(inner.handle)
        };
    }
}
