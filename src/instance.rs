use lilv_sys as lib;
use lv2_raw::core::LV2Descriptor;
use lv2_raw::core::LV2Handle;
use std::ffi::CStr;
use std::ptr::NonNull;

#[repr(C)]
pub(crate) struct InstanceImpl {
    pub descriptor: *const LV2Descriptor,
    pub handle: LV2Handle,
    pub private: *mut (),
}

pub struct Instance {
    pub(crate) inner: NonNull<InstanceImpl>,
}

unsafe impl Send for Instance {}

impl InstanceImpl {
    #[inline(always)]
    pub fn uri(&self) -> Option<&str> {
        unsafe { CStr::from_ptr((*self.descriptor).uri).to_str().ok() }
    }

    #[inline(always)]
    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        if port_index < std::u32::MAX as _ {
            ((*self.descriptor).connect_port)(self.handle, port_index as _, data as *mut _ as _)
        }
    }

    #[inline(always)]
    pub fn activate(&mut self) {
        if let Some(activate) = unsafe { (*self.descriptor).activate } {
            activate(self.handle)
        }
    }

    #[inline(always)]
    pub fn run(&mut self, sample_count: usize) {
        let run = unsafe { (*self.descriptor).run };
        let mut sc = sample_count;

        while sc != 0 {
            let n = sc.min(std::u32::MAX as _);
            run(self.handle, n as _);
            sc -= n;
        }
    }

    #[inline(always)]
    pub fn deactivate(&mut self) {
        if let Some(deactivate) = unsafe { (*self.descriptor).deactivate } {
            deactivate(self.handle)
        }
    }

    #[inline(always)]
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        let uri_c = crate::make_c_string(uri);
        let uri = crate::choose_string(uri, &uri_c);

        NonNull::new((self.descriptor().extension_data)(uri as _) as _)
    }

    #[inline(always)]
    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { std::mem::transmute(self.descriptor) }
    }

    #[inline(always)]
    pub fn handle(&self) -> LV2Handle {
        self.handle
    }
}

impl Instance {
    #[inline(always)]
    pub fn uri(&self) -> Option<&str> {
        unsafe { self.inner.as_ref().uri() }
    }

    #[inline(always)]
    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        self.inner.as_mut().connect_port(port_index, data)
    }

    #[inline(always)]
    pub fn activate(&mut self) {
        unsafe { self.inner.as_mut().activate() }
    }

    #[inline(always)]
    pub fn run(&mut self, sample_count: usize) {
        unsafe { self.inner.as_mut().run(sample_count) }
    }

    #[inline(always)]
    pub fn deactivate(&mut self) {
        unsafe { self.inner.as_mut().deactivate() }
    }

    #[inline(always)]
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        self.inner.as_ref().extension_data(uri)
    }

    #[inline(always)]
    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { self.inner.as_ref().descriptor() }
    }

    #[inline(always)]
    pub fn handle(&self) -> LV2Handle {
        unsafe { self.inner.as_ref().handle() }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { lib::lilv_instance_free(self.inner.as_ptr() as _) };
    }
}
