use lilv_sys as lib;
use lv2_raw::core::LV2Descriptor;
use lv2_raw::core::LV2Handle;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct Instance {
    pub(crate) inner: NonNull<lib::LilvInstanceImpl>,
    pub(crate) active: bool,
}

pub struct ActiveInstance {
    pub(crate) inner: Instance,
}

unsafe impl Send for Instance {}

impl Instance {
    #[must_use]
    pub fn uri(&self) -> Option<&str> {
        unsafe {
            CStr::from_ptr((*self.inner.as_ref().lv2_descriptor).uri)
                .to_str()
                .ok()
        }
    }

    /// # Safety
    /// Connecting a port calls a plugin's code, which itself may be unsafe.
    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        if port_index >= std::u32::MAX as _ {
            return;
        }
        let data_ptr: *mut T = data;
        ((*self.inner.as_ref().lv2_descriptor).connect_port)(
            self.inner.as_ref().lv2_handle,
            port_index as u32,
            data_ptr.cast(),
        );
    }

    /// # Safety
    /// Calling external code may be unsafe.
    pub unsafe fn activate(self) -> Option<ActiveInstance> {
        let activate_fn = (*self.inner.as_ref().lv2_descriptor).activate?;
        activate_fn(self.inner.as_ref().lv2_handle);
        let mut inner = self;
        inner.active = true;
        Some(ActiveInstance { inner })
    }

    /// # Safety
    /// Gathering extension data call's a plugins code, which itself may be unsafe.
    #[must_use]
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        let uri = std::ffi::CString::new(uri).ok()?;
        NonNull::new((self.descriptor().extension_data)(uri.as_ptr().cast()) as _)
    }

    #[must_use]
    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { self.inner.as_ref().lv2_descriptor.as_ref().unwrap() }
    }

    #[must_use]
    pub fn handle(&self) -> LV2Handle {
        unsafe { self.inner.as_ref().lv2_handle }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if self.active {
            if let Some(deactivate_fn) = unsafe { (*self.inner.as_ref().lv2_descriptor).deactivate }
            {
                unsafe { deactivate_fn(self.inner.as_ref().lv2_handle) };
                self.active = false;
            }
        }
        unsafe { lib::lilv_instance_free(self.inner.as_ptr().cast()) };
    }
}

impl ActiveInstance {
    /// # Safety
    /// Calling external code may be unsafe.
    #[allow(clippy::cast_possible_truncation)]
    pub unsafe fn run(&mut self, sample_count: usize) {
        let run = (*self.inner.inner.as_ref().lv2_descriptor).run;
        let mut sc = sample_count;

        while sc != 0 {
            let n = sc.min(std::u32::MAX as _);
            run(self.inner.inner.as_ref().lv2_handle, n as _);
            sc -= n;
        }
    }

    /// # Safety
    /// Calling external code may be unsafe.
    pub unsafe fn deactivate(self) -> Option<Instance> {
        let deactivate_fn = (*self.inner.inner.as_ref().lv2_descriptor).deactivate?;
        deactivate_fn(self.inner.inner.as_ref().lv2_handle);
        let mut inner = self.inner;
        inner.active = false;
        Some(inner)
    }
}
