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
    pub fn uri(&self) -> Option<&str> {
        unsafe { CStr::from_ptr((*self.descriptor).uri).to_str().ok() }
    }

    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        if port_index >= std::u32::MAX as _ {
            return;
        }
        ((*self.descriptor).connect_port)(self.handle, port_index as _, data as *mut _ as _);
    }

    pub fn activate(&mut self) {
        if let Some(activate) = unsafe { (*self.descriptor).activate } {
            activate(self.handle);
        }
    }

    pub fn run(&mut self, sample_count: usize) {
        let run = unsafe { (*self.descriptor).run };
        let mut sc = sample_count;

        while sc != 0 {
            let n = sc.min(std::u32::MAX as _);
            run(self.handle, n as _);
            sc -= n;
        }
    }

    pub fn deactivate(&mut self) {
        if let Some(deactivate) = unsafe { (*self.descriptor).deactivate } {
            deactivate(self.handle);
        }
    }

    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        let uri_c = crate::make_c_string(uri);
        let uri = crate::choose_string(uri, &uri_c);

        NonNull::new((self.descriptor().extension_data)(uri as _) as _)
    }

    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { &*self.descriptor }
    }

    pub fn handle(&self) -> LV2Handle {
        self.handle
    }
}

impl Instance {
    pub fn uri(&self) -> Option<&str> {
        unsafe { self.inner.as_ref().uri() }
    }

    /// # Safety
    /// Connecting a port calls a plugin's code, which itself may be unsafe.
    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        self.inner.as_mut().connect_port(port_index, data)
    }

    pub fn activate(&mut self) {
        unsafe { self.inner.as_mut().activate() }
    }

    pub fn run(&mut self, sample_count: usize) {
        unsafe { self.inner.as_mut().run(sample_count) }
    }

    pub fn deactivate(&mut self) {
        unsafe { self.inner.as_mut().deactivate() }
    }

    /// # Safety
    /// Gathering extension data call's a plugins code, which itself may be unsafe.
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        self.inner.as_ref().extension_data(uri)
    }

    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { self.inner.as_ref().descriptor() }
    }

    pub fn handle(&self) -> LV2Handle {
        unsafe { self.inner.as_ref().handle() }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { lib::lilv_instance_free(self.inner.as_ptr() as _) };
    }
}
