use lilv_sys as lib;
use lv2_raw::core::LV2Descriptor;
use lv2_raw::core::LV2Handle;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ptr::NonNull;

/// An LV2 plugin instance.
#[allow(clippy::module_name_repetitions)]
pub struct Instance {
    pub(crate) inner: NonNull<lib::LilvInstanceImpl>,
}

/// An LV2 plugin instance that has been activated and is ready to process data.
#[allow(clippy::module_name_repetitions)]
pub struct ActiveInstance {
    pub(crate) inner: Instance,
}

unsafe impl Send for Instance {}

impl Instance {
    /// Returns the URI of the plugin for the instance.
    /// This is a globally unique string for the plugin.
    #[must_use]
    pub fn uri(&self) -> Option<&str> {
        unsafe {
            CStr::from_ptr((*self.inner.as_ref().lv2_descriptor).uri)
                .to_str()
                .ok()
        }
    }

    /// Connect a port on a plugin instance to a memory location.
    ///
    /// Plugin writers should be aware that the host may elect to use the same
    /// buffer for more than one port and even use the same buffer for both
    /// input and output (see lv2:inPlaceBroken in lv2.ttl).
    ///
    /// If the plugin has the feature lv2:hardRTCapable then there are various
    /// things that the plugin MUST NOT do within the `connect_port()` function;
    /// see lv2core.ttl for details.
    ///
    /// `connect_port()` MUST be called at least once for each port before
    /// `run()` is called, unless that port is lv2:connectionOptional. The
    /// plugin must pay careful attention to the block size passed to run()
    /// since the block allocated may only just be large enough to contain the
    /// data, and is not guaranteed to remain constant between run() calls.
    ///
    /// `connect_port()` may be called more than once for a plugin instance to
    /// allow the host to change the buffers that the plugin is reading or
    /// writing.
    ///
    /// The host MUST NOT try to connect a `port_index` that is not defined in
    /// the plugin's RDF data. If it does, the plugin's behaviour is undefined
    /// (a crash is likely).
    ///
    /// `data` should point to data of the type defined by the port type in the
    /// plugin's RDF data (e.g. an array of float for an lv2:AudioPort). This
    /// pointer must be stored by the plugin instance and used to read/write
    /// data when run() is called. Data present at the time of the
    /// `connect_port()` call MUST NOT be considered meaningful.
    ///
    /// # Safety
    /// Connecting a port calls a plugin's code, which itself may be unsafe.
    pub unsafe fn connect_port<T>(&mut self, port_index: usize, data: &mut T) {
        let port_index = match u32::try_from(port_index) {
            Ok(port_index) => port_index,
            Err(_) => return,
        };
        let data_ptr: *mut T = data;
        ((*self.inner.as_ref().lv2_descriptor).connect_port)(
            self.inner.as_ref().lv2_handle,
            port_index as u32,
            data_ptr.cast(),
        );
    }

    /// Activate a plugin instance.
    ///
    /// This resets all state information in the plugin except for port
    /// connections.
    ///
    /// # Safety
    /// Calling external code may be unsafe.
    #[must_use]
    pub unsafe fn activate(self) -> Option<ActiveInstance> {
        let activate_fn = (*self.inner.as_ref().lv2_descriptor).activate?;
        activate_fn(self.inner.as_ref().lv2_handle);
        Some(ActiveInstance { inner: self })
    }

    /// Get the extension data for a plugin instance.
    ///
    /// The type and semantics of the data returned is specific to the
    /// particular extension, though in all cases it is shared and must not be
    /// deleted.
    ///
    /// # Safety
    /// Gathering extension data call's a plugins code, which itself may be unsafe.
    #[must_use]
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        let uri = std::ffi::CString::new(uri).ok()?;
        NonNull::new(
            ((*(self.inner.as_ref().lv2_descriptor)).extension_data)(uri.as_ptr().cast()) as _,
        )
    }

    /// Get the raw descriptor for the plugin.
    #[must_use]
    pub fn descriptor(&self) -> Option<&LV2Descriptor> {
        unsafe { self.inner.as_ref().lv2_descriptor.as_ref() }
    }

    /// Get the raw handle for the plugin instance.
    #[must_use]
    pub fn handle(&self) -> LV2Handle {
        unsafe { self.inner.as_ref().lv2_handle }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { lib::lilv_instance_free(self.inner.as_ptr().cast()) };
    }
}

impl ActiveInstance {
    /// Run the plugin instance for `sample_count` frames.
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

    /// Deactivate the plugin instance.
    ///
    /// Note: This will reset all state information except for port connections.
    ///
    /// # Safety
    /// Calling external code may be unsafe.
    #[must_use]
    pub unsafe fn deactivate(self) -> Option<Instance> {
        let mut active_instance = self;
        let instance = active_instance
            .deactive_impl()
            .map(|i| Instance { inner: i })?;
        // Prevent running deactivate twice since we manually called the drop
        // side-effects with `deactivate_impl`..
        std::mem::forget(active_instance);
        Some(instance)
    }

    /// Get the underlying instance.
    ///
    /// This is useful to call `connect_port` if the data locations have changed.
    #[must_use]
    pub fn instance(&self) -> &Instance {
        &self.inner
    }

    fn deactive_impl(&mut self) -> Option<NonNull<lib::LilvInstanceImpl>> {
        let deactivate_fn = unsafe { (*self.inner.inner.as_ref().lv2_descriptor).deactivate }?;
        unsafe { deactivate_fn(self.inner.inner.as_ref().lv2_handle) };
        Some(self.inner.inner)
    }
}

impl Drop for ActiveInstance {
    fn drop(&mut self) {
        self.deactive_impl();
    }
}
