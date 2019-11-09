use lilv_sys::*;
use lv2_raw::{LV2Descriptor, LV2Handle};
use std::convert::{AsMut, AsRef};
use std::ffi::CStr;

/// An instance of an LV2 plugin.
pub struct Instance(*mut LilvInstance);

/// An instance of an LV2 plugin that is ready for running.
///
/// It is created by calling `activate` on an `Instance`.
pub struct ActiveInstance(Instance);

unsafe impl Send for Instance {}
unsafe impl Send for ActiveInstance {}

impl Instance {
    /// Create a new instance from a raw pointer.
    pub unsafe fn from_raw(raw: *mut LilvInstance) -> Instance {
        Instance(raw)
    }

    /// Return the raw pointer for this underlying instance.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut LilvInstance {
        self.0
    }

    /// The descriptor for the LV2 plugin.
    #[inline(always)]
    pub fn descriptor(&self) -> &LV2Descriptor {
        unsafe { (*self.0).lv2_descriptor.as_ref().unwrap() }
    }

    /// The handle for the LV2 plugin.
    #[inline(always)]
    pub fn handle(&self) -> LV2Handle {
        unsafe { (*self.0).lv2_handle }
    }

    /// The uri for the plugin.
    #[inline(always)]
    pub fn uri(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.descriptor().uri) }
    }

    /// Connect the port with the given index to the given data.
    #[inline(always)]
    pub unsafe fn connect_port<T>(&mut self, port_index: u32, data: *mut T) {
        (self.descriptor().connect_port)(self.handle(), port_index, data as *mut core::ffi::c_void)
    }

    /// Activate the plugin so that it can be run.
    #[inline(always)]
    pub fn activate(self) -> ActiveInstance {
        if let Some(activate) = self.descriptor().activate {
            activate(self.handle())
        }
        ActiveInstance(self)
    }

    /// The extension data for uri from the plugin.
    #[inline(always)]
    pub unsafe fn extension_data(&self, uri: &CStr) -> *const libc::c_void {
        let f = self.descriptor().extension_data;
        f(uri.as_ptr() as *const u8)
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        let descriptor = self.descriptor();
        (descriptor.cleanup)(self.handle())
    }
}

impl ActiveInstance {
    /// Get the underlying instance of the plugin.
    #[inline(always)]
    pub fn as_instance(&self) -> &Instance {
        &self.0
    }

    /// Get the underlying instance of the plugin.
    #[inline(always)]
    pub fn as_mut_instance(&mut self) -> &mut Instance {
        &mut self.0
    }

    /// Run the plugin for the given amount of frames.
    #[inline(always)]
    pub fn run(&mut self, sample_count: u32) {
        (self.as_instance().descriptor().run)(self.as_instance().handle(), sample_count)
    }

    /// Deactivate the plugin and return the underlying instance.
    ///
    /// This is not required as the instance will call deactivate on drop.
    pub fn deactivate(self) -> Instance {
        let mut mut_self = self;
        // We pass along the pointer and rely on `ActiveInstance`s drop implementation
        // to deactivate.
        Instance(mut_self.0.as_mut_ptr())
    }
}

impl AsRef<Instance> for ActiveInstance {
    fn as_ref(&self) -> &Instance {
        self.as_instance()
    }
}

impl AsMut<Instance> for ActiveInstance {
    fn as_mut(&mut self) -> &mut Instance {
        self.as_mut_instance()
    }
}

impl Drop for ActiveInstance {
    fn drop(&mut self) {
        if let Some(deactivate) = self.as_instance().descriptor().deactivate {
            deactivate(self.as_instance().handle())
        }
    }
}
