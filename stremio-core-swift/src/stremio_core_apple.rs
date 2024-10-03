use std::panic;
use std::sync::RwLock;

use futures::{future, StreamExt};

/*
TODO: For some reason obj-c objects that generated from rust are not deallocated by swift automaitcly,
As a work around im deallocating NSData manually and casting from NSObject to prevent swift to convert it Data
Do not use NSData argument for swift as swift enforces Swift's Data type which will cause problems at some point
*/
use objc2::ffi::{objc_object, objc_release, Nil};
use objc2::rc::Retained;
use objc2::{class, msg_send};
use objc2_foundation::{NSData, NSObject, NSString};

use enclose::enclose;
use once_cell::sync::{Lazy, OnceCell};
use prost::Message;

use stremio_core::constants::{
    DISMISSED_EVENTS_STORAGE_KEY, LIBRARY_RECENT_STORAGE_KEY, LIBRARY_STORAGE_KEY,
    NOTIFICATIONS_STORAGE_KEY, PROFILE_STORAGE_KEY, SEARCH_HISTORY_STORAGE_KEY,
    STREAMS_STORAGE_KEY,
};
use stremio_core::models::common::Loadable;
use stremio_core::runtime::{Env, EnvError, Runtime, RuntimeEvent};
use stremio_core::types::events::DismissedEventsBucket;
use stremio_core::types::library::LibraryBucket;
use stremio_core::types::notifications::NotificationsBucket;
use stremio_core::types::profile::Profile;
use stremio_core::types::resource::Stream;
use stremio_core::types::search_history::SearchHistoryBucket;
use stremio_core::types::streams::StreamsBucket;

use crate::env::{AppleEnv, AppleEvent};
use crate::model::AppleModel;
use stremio_core_protobuf::{
    bridge::{FromProtobuf, ToProtobuf},
    protobuf::stremio::core::runtime::{self, Field},
};

static RUNTIME: Lazy<RwLock<Option<Loadable<Runtime<AppleEnv, AppleModel>, EnvError>>>> =
    Lazy::new(|| Default::default());

/// The device name passed on [`initializeNative`] of `stremio-core-swift`.
pub static DEVICE_NAME: OnceCell<String> = OnceCell::new();

#[no_mangle]
pub extern "C" fn initialize_rust() {
    panic::set_hook(Box::new(|info| {
        let stremio_core_class = class!(_TtC11StremioCore4Core);
        let string = NSString::from_str(info.to_string().as_str());
        let _: () = unsafe { msg_send![stremio_core_class, onRustPanic: string.as_ref()] };
    }));
}

#[no_mangle]
pub unsafe extern "C" fn initializeNative(device_info: *mut NSString) -> *mut NSObject {
    let init_result = AppleEnv::exec_sync(AppleEnv::init());

    // Set the device name only once on initialization!
    DEVICE_NAME
        .set(unsafe { &*device_info }.to_string())
        .expect("Device name should be set only once!");

    match init_result {
        Ok(_) => {
            let storage_result = AppleEnv::exec_sync(future::try_join3(
                future::try_join5(
                    AppleEnv::get_storage::<Profile>(PROFILE_STORAGE_KEY),
                    AppleEnv::get_storage::<LibraryBucket>(LIBRARY_RECENT_STORAGE_KEY),
                    AppleEnv::get_storage::<LibraryBucket>(LIBRARY_STORAGE_KEY),
                    AppleEnv::get_storage::<StreamsBucket>(STREAMS_STORAGE_KEY),
                    AppleEnv::get_storage::<NotificationsBucket>(NOTIFICATIONS_STORAGE_KEY),
                ),
                AppleEnv::get_storage::<SearchHistoryBucket>(SEARCH_HISTORY_STORAGE_KEY),
                AppleEnv::get_storage::<DismissedEventsBucket>(DISMISSED_EVENTS_STORAGE_KEY),
            ));
            match storage_result {
                Ok((
                    (profile, recent_bucket, other_bucket, streams, notifications),
                    search_history,
                    dismissed_events,
                )) => {
                    let profile = profile.unwrap_or_default();
                    let mut library = LibraryBucket::new(profile.uid(), vec![]);
                    if let Some(recent_bucket) = recent_bucket {
                        library.merge_bucket(recent_bucket);
                    };
                    if let Some(other_bucket) = other_bucket {
                        library.merge_bucket(other_bucket);
                    };
                    let streams = streams.unwrap_or(StreamsBucket::new(profile.uid()));
                    let notifications = notifications
                        .unwrap_or(NotificationsBucket::new::<AppleEnv>(profile.uid(), vec![]));
                    let search_history =
                        search_history.unwrap_or(SearchHistoryBucket::new(profile.uid()));
                    let dismissed_events =
                        dismissed_events.unwrap_or(DismissedEventsBucket::new(profile.uid()));
                    let (model, effects) = AppleModel::new(
                        profile,
                        library,
                        streams,
                        notifications,
                        search_history,
                        dismissed_events,
                    );
                    let (runtime, rx) = Runtime::<AppleEnv, _>::new(
                        model,
                        effects.into_iter().collect::<Vec<_>>(),
                        1000,
                    );
                    let stremio_core_class = class!(_TtC11StremioCore4Core);
                    AppleEnv::exec_concurrent(rx.for_each(move |event| {
                        if let RuntimeEvent::CoreEvent(event) = &event {
                            AppleEnv::exec_concurrent(enclose!((event) async move {
                                let runtime = RUNTIME.read().expect("runtime read failed");
                                let runtime = runtime
                                    .as_ref()
                                    .expect("runtime is not ready")
                                    .as_ref()
                                    .expect("runtime is not ready");
                                let model = runtime.model().expect("model read failed");
                                AppleEnv::emit_to_analytics(
                                    &AppleEvent::CoreEvent(event.to_owned()),
                                    &model,
                                    "TODO"
                                );
                            }));
                        };
                        let eventbytes =
                            &NSData::with_bytes(&event.to_protobuf::<AppleEnv>(&()).encode_to_vec());
                        let _: () = unsafe {
                            msg_send![stremio_core_class, onRuntimeEvent: eventbytes.as_ref()]
                        };
                        future::ready(())
                    }));
                    *RUNTIME.write().expect("RUNTIME write failed") =
                        Some(Loadable::Ready(runtime));
                    Nil as *mut NSObject
                }
                Err(error) => {
                    *RUNTIME.write().expect("RUNTIME write failed") =
                        Some(Loadable::Err(error.to_owned()));
                    let result_bytes = error.to_protobuf::<AppleEnv>(&()).encode_to_vec();
                    Retained::into_raw(NSData::with_bytes(result_bytes.as_ref())) as *mut NSObject
                }
            }
        }
        Err(error) => {
            *RUNTIME.write().expect("RUNTIME write failed") = Some(Loadable::Err(error.to_owned()));
            let result_bytes = error.to_protobuf::<AppleEnv>(&()).encode_to_vec();
            Retained::into_raw(NSData::with_bytes(result_bytes.as_ref())) as *mut NSObject
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn dispatchNative(action_protobuf: *mut NSObject) {
    // Convert the incoming action_protobuf bytes to a Vec<u8>
    let action_bytes = unsafe { &*(action_protobuf as *mut NSData) }.bytes();
    let runtime_action = match runtime::RuntimeAction::decode(action_bytes) {
        Ok(action) => action.from_protobuf(),
        Err(err) => {
            eprintln!("Error decoding RuntimeAction protobuf: {:?}", err);
            return;
        }
    };
    let runtime = RUNTIME.read().expect("RUNTIME read failed");
    let runtime = runtime
        .as_ref()
        .expect("RUNTIME not initialized")
        .as_ref()
        .expect("RUNTIME not initialized");
    runtime.dispatch(runtime_action);
}

#[no_mangle]
pub unsafe extern "C" fn getStateNative(field: i32) -> *mut NSObject {
    let field = Field::try_from(field)
        .ok()
        .from_protobuf()
        .expect("AppleModelField convert failed");
    let runtime = RUNTIME.read().expect("RUNTIME read failed");
    let runtime = runtime
        .as_ref()
        .expect("RUNTIME not initialized")
        .as_ref()
        .expect("RUNTIME not initialized");
    let model = runtime.model().expect("model read failed");
    let bytes: &[u8] = &model.get_state_binary(&field);
    Retained::into_raw(NSData::with_bytes(bytes)) as *mut NSObject
}

//Returns 0 address as Null
#[no_mangle]
pub unsafe extern "C" fn decodeStreamDataNative(field: *mut NSString) -> *mut NSObject {
    let field = &*field;
    let stream = match Stream::decode(field.to_string()) {
        Ok(stream) => stream
            .to_protobuf::<AppleEnv>(&(None, None, None, None))
            .encode_to_vec(),
        Err(_) => return Nil as *mut NSObject,
    };
    Retained::into_raw(NSData::with_bytes(stream.as_ref())) as *mut NSObject
}

#[no_mangle]
pub unsafe extern "C" fn sendNextAnalyticsBatch() {
    AppleEnv::exec_concurrent(AppleEnv::send_next_analytics_batch());
}

#[no_mangle]
pub extern "C" fn getVersionNative() -> *mut NSString {
    Retained::into_raw(NSString::from_str(env!("CARGO_PKG_VERSION")))
}

#[no_mangle]
pub unsafe extern "C" fn releaseObjectNative(object: *mut NSObject) {
    objc_release(object as *mut objc_object);
}
