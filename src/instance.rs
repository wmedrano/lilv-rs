use lilv_sys as lib;
use lv2_raw::core::LV2Descriptor;
use lv2_raw::core::LV2Handle;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ptr::NonNull;

#[allow(clippy::module_name_repetitions)]
pub struct Instance {
    pub(crate) inner: NonNull<lib::LilvInstanceImpl>,
}

#[allow(clippy::module_name_repetitions)]
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

    /// # Safety
    /// Calling external code may be unsafe.
    #[must_use]
    pub unsafe fn activate(self) -> Option<ActiveInstance> {
        let activate_fn = (*self.inner.as_ref().lv2_descriptor).activate?;
        activate_fn(self.inner.as_ref().lv2_handle);
        Some(ActiveInstance { inner: self })
    }

    /// # Safety
    /// Gathering extension data call's a plugins code, which itself may be unsafe.
    #[must_use]
    pub unsafe fn extension_data<T>(&self, uri: &str) -> Option<NonNull<T>> {
        let uri = std::ffi::CString::new(uri).ok()?;
        NonNull::new(
            ((*(self.inner.as_ref().lv2_descriptor)).extension_data)(uri.as_ptr().cast()) as _,
        )
    }

    #[must_use]
    pub fn descriptor(&self) -> Option<&LV2Descriptor> {
        unsafe { self.inner.as_ref().lv2_descriptor.as_ref() }
    }

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

// #[cfg(test)]
// mod tests {
//     use std::collections::HashSet;

//     use crate::*;

//     const SAMPLE_RATE: f64 = 44100.0;

//     #[test]
//     fn test_activate_all_plugins() {
//         let world = World::with_load_all();
//         let have_features = HashSet::<String>::new();
//         for plugin in world.plugins() {
//             let plugin_name = plugin.name().as_str().unwrap().to_string();
//             let required_features = plugin.required_features();
//             for feature in required_features.iter() {
//                 let feature_name = feature.as_str().unwrap();
//                 if !have_features.contains(feature_name) {
//                     continue;
//                 }
//             }
//             let instance = match unsafe { plugin.instantiate(SAMPLE_RATE, &[]) } {
//                 Some(i) => i,
//                 None => {
//                     println!("{}: Failed to instantiate.", plugin_name);
//                     continue;
//                 }
//             };
//             let active_instance = match unsafe { instance.activate() } {
//                 Some(i) => i,
//                 None => {
//                     println!("{}: Failed to activate.", plugin_name);
//                     continue;
//                 }
//             };
//             unsafe { active_instance.deactivate().unwrap() };
//         }
//     }
// }
