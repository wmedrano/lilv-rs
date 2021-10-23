use log::{error, info};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::sync::atomic::AtomicU32;
use std::sync::RwLock;

/// An implementation of LV2 URID map.
enum UridMapFeatureImpl<'a> {
    /// A native Rust implementation.
    Native(Box<UridMapFeatureNativeImpl>),
    /// An abstract implementation exposed through the LV2_URID_Map handle and function pointer.
    Abstract(&'a lv2_raw::LV2UridMap),
}

impl<'a> UridMapFeatureImpl<'a> {
    fn map(&self, uri: &CStr) -> u32 {
        match self {
            UridMapFeatureImpl::Native(f) => f.map(uri),
            UridMapFeatureImpl::Abstract(f) => {
                let handle = f.handle;
                (f.map)(handle, uri.as_ptr())
            }
        }
    }
}

/// Provides the urid map feature for LV2. See documentation for urid map at
/// http://lv2plug.in/ns/ext/urid/#map.
// The fields are actually referenced as void ptrs within feature and data.
#[allow(dead_code)]
pub struct UridMapFeature<'a> {
    feature: lv2_raw::LV2Feature,
    data: Option<Box<lv2_raw::LV2UridMap>>,
    urid_map_impl: UridMapFeatureImpl<'a>,
}

unsafe impl Send for UridMapFeature<'static> {}
unsafe impl Sync for UridMapFeature<'static> {}

static LV2_URID__MAP: &[u8] = b"http://lv2plug.in/ns/ext/urid#map\0";

impl Default for UridMapFeature<'static> {
    /// Create the default instance for UridMapFeature with no registered URIs. URIs will register
    /// themselves with the `get` method.
    fn default() -> UridMapFeature<'static> {
        let mut urid_map_impl: Box<UridMapFeatureNativeImpl> = Box::default();
        let mut data = Box::new(lv2_raw::LV2UridMap {
            handle: urid_map_impl.as_mut() as *mut UridMapFeatureNativeImpl
                as *mut std::ffi::c_void,
            map: urid_map_feature_native_impl_map,
        });
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: LV2_URID__MAP.as_ptr() as *const ::std::os::raw::c_char,
                data: data.as_mut() as *mut lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: Some(data),
            urid_map_impl: UridMapFeatureImpl::Native(urid_map_impl),
        }
    }
}

extern "C" fn urid_map_feature_native_impl_map(
    handle: *mut std::ffi::c_void,    /*Type is UridMapFeatureNativeImpl*/
    uri: *const std::os::raw::c_char, /*CStr*/
) -> u32 {
    let self_ptr = handle as *const UridMapFeatureNativeImpl;
    unsafe {
        match self_ptr.as_ref() {
            Some(self_ref) => self_ref.map(CStr::from_ptr(uri)),
            None => {
                error!("URID Map had null handle for UridMapFeatureNativeImpl.");
                0
            }
        }
    }
}

impl<'a> UridMapFeature<'a> {
    /// The URI for the urid map LV2 feature.
    pub const URI: &'static str = "http://lv2plug.in/ns/ext/urid#map";

    /// Get the urid map as an LV2_feature.
    pub fn as_lv2_feature_mut(&mut self) -> &mut lv2_raw::LV2Feature {
        &mut self.feature
    }

    /// Returns `true` if this instance is backed by a native implementation. `false` is returned
    /// if it is implemented behind opaque pointers within LV2_Feature.
    pub fn is_native(&self) -> bool {
        matches!(&self.urid_map_impl, UridMapFeatureImpl::Native(_))
    }

    /// Get the id for the given uri. If the uri does not have an ID, it will be registered
    /// with a new one.
    ///
    /// Note: This method makes uses of mutexes and heap based maps; do not run in a realtime
    /// context. If needed, cache the returned IDs.
    pub fn map(&self, uri: &CStr) -> u32 {
        self.urid_map_impl.map(uri)
    }
}

impl<'a> From<&'a lv2_raw::LV2UridMap> for UridMapFeature<'a> {
    fn from(map: &'a lv2_raw::LV2UridMap) -> UridMapFeature<'a> {
        UridMapFeature {
            feature: lv2_raw::LV2Feature {
                uri: lv2_raw::LV2_URID__MAP.as_ptr() as *const ::std::os::raw::c_char,
                data: map as *const lv2_raw::LV2UridMap as *mut std::ffi::c_void,
            },
            data: None, /*The data is borrowed from map*/
            urid_map_impl: UridMapFeatureImpl::Abstract(map),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UridFeatureError {
    FeatureDataIsNull,
    FeatureIsNotUridMap,
}

impl<'a> TryFrom<&'a mut lv2_raw::LV2Feature> for UridMapFeature<'a> {
    type Error = UridFeatureError;

    /// Convert the feature into a UridMapFeature. If the LV2 feature is not a URID map feature,
    /// then an error is returned.
    fn try_from(
        feature: &'a mut lv2_raw::LV2Feature,
    ) -> Result<UridMapFeature<'a>, UridFeatureError> {
        let feature_uri = unsafe { CStr::from_ptr(feature.uri) };
        if feature_uri.to_bytes() == lv2_raw::LV2_URID__MAP.as_bytes() {
            let urid_map_ptr = feature.data as *const lv2_raw::LV2UridMap;
            match unsafe { urid_map_ptr.as_ref() } {
                Some(r) => Ok(UridMapFeature::from(r)),
                None => Err(UridFeatureError::FeatureDataIsNull),
            }
        } else {
            Err(UridFeatureError::FeatureIsNotUridMap)
        }
    }
}

/// Implementation for uri map LV2 feature.
struct UridMapFeatureNativeImpl {
    map: RwLock<HashMap<CString, u32>>,
    next_id: AtomicU32,
}

impl Default for UridMapFeatureNativeImpl {
    /// Create a new UridMapFeatureNativeImpl. With no registered URIs.
    fn default() -> UridMapFeatureNativeImpl {
        UridMapFeatureNativeImpl {
            map: RwLock::default(),
            next_id: AtomicU32::new(1),
        }
    }
}

impl UridMapFeatureNativeImpl {
    /// Get the ID for the given uri. If the URI is not registered, then it will be registered
    /// with a new unique ID. This function makes use of a heap based hash map and mutex so it is
    /// not suitable for realtime execution. Results for important URIs should be cached.
    fn map(&self, uri: &CStr) -> u32 {
        if let Some(id) = self.map.read().unwrap().get(uri).copied() {
            return id;
        };
        let mut map = self.map.write().unwrap();
        // We check if the ID is present again in case it was inserted in the time between
        // releasing the read lock and regaining the write lock.
        if let Some(id) = map.get(uri).copied() {
            return id;
        }
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        info!("Mapped URI {:?} to {}.", uri, id);
        map.insert(CString::from(uri), id);
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urid_maps_to_same_value_for_same_uri() {
        let m = UridMapFeature::default();
        let a = m.map(CStr::from_bytes_with_nul(b"a\0").unwrap());
        let b = m.map(CStr::from_bytes_with_nul(b"a\0").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn urid_map_maps_to_unique_values() {
        let m = UridMapFeature::default();
        let a = m.map(CStr::from_bytes_with_nul(b"a\0").unwrap());
        let b = m.map(CStr::from_bytes_with_nul(b"b\0").unwrap());
        assert_ne!(a, b);
    }

    #[test]
    fn urid_map_can_be_made_from_lv2_feature() {
        let mut native_impl = UridMapFeature::default();
        let native_impl_mapping = native_impl.map(CStr::from_bytes_with_nul(b"a\0").unwrap());
        assert!(native_impl.is_native());

        let abstract_impl = UridMapFeature::try_from(native_impl.as_lv2_feature_mut()).unwrap();
        let abstract_impl_mapping = abstract_impl.map(CStr::from_bytes_with_nul(b"a\0").unwrap());
        assert!(!abstract_impl.is_native());

        assert_eq!(
            native_impl_mapping, abstract_impl_mapping,
            "Expected native and abstract impls to give the same result."
        );
    }

    #[test]
    fn urid_map_from_non_urid_map_feature_returns_error() {
        let mut data = 1;
        let valid_ptr: *mut i32 = &mut data;
        let bad_uri = CStr::from_bytes_with_nul(b"bad_uri\0").unwrap();
        assert_eq!(
            UridMapFeature::try_from(&mut lv2_raw::LV2Feature {
                uri: bad_uri.as_ptr(),
                data: valid_ptr as *mut std::ffi::c_void,
            })
            .err(),
            Some(UridFeatureError::FeatureIsNotUridMap)
        );
    }

    #[test]
    fn urid_map_from_feature_with_null_ptr_returns_error() {
        let uri = CString::new(UridMapFeature::URI).unwrap();
        assert_eq!(
            UridMapFeature::try_from(&mut lv2_raw::LV2Feature {
                uri: uri.as_ptr(),
                data: std::ptr::null_mut(),
            })
            .err(),
            Some(UridFeatureError::FeatureDataIsNull)
        );
    }
}
