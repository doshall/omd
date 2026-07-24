use wasm_bindgen::prelude::*;

pub const STORAGE_VERSION_KEY: &str = "omd-web-storage-version";
pub const STORAGE_VERSION_IDB: &str = "idb";

pub fn storage_uses_idb() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        return false;
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::tabs::load_storage(STORAGE_VERSION_KEY).as_deref() == Some(STORAGE_VERSION_IDB)
    }
}

pub fn enable_idb_storage() {
    crate::tabs::save_storage(STORAGE_VERSION_KEY, STORAGE_VERSION_IDB);
}

pub fn tab_key(id: &str) -> String {
    format!("tab:{id}")
}

pub fn recent_key(id: &str) -> String {
    format!("recent:{id}")
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use js_sys::Promise;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{IdbDatabase, IdbOpenDbRequest, IdbRequest, IdbTransaction, IdbTransactionMode};

    const DB_NAME: &str = "omd-web";
    const STORE_NAME: &str = "documents";
    const DB_VERSION: u32 = 1;

    fn object_store_exists(db: &IdbDatabase) -> bool {
        let names = db.object_store_names();
        for index in 0..names.length() {
            if names.item(index).as_deref() == Some(STORE_NAME) {
                return true;
            }
        }
        false
    }

    fn request_promise(request: &IdbRequest) -> Promise {
        let request = request.clone();
        Promise::new(&mut |resolve, reject| {
            let resolve = resolve.clone();
            let reject = reject.clone();
            let success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let target = event.target().unwrap();
                let request: IdbRequest = target.dyn_into().unwrap();
                let result = request.result().unwrap_or(JsValue::UNDEFINED);
                let _ = resolve.call1(&JsValue::NULL, &result);
            }) as Box<dyn FnMut(_)>);
            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            success.forget();

            let error = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("idb request failed"));
            }) as Box<dyn FnMut(_)>);
            request.set_onerror(Some(error.as_ref().unchecked_ref()));
            error.forget();
        })
    }

    fn open_request_promise(request: &IdbOpenDbRequest) -> Promise {
        let request = request.clone();
        Promise::new(&mut |resolve, reject| {
            let resolve = resolve.clone();
            let reject = reject.clone();
            let success = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let target = event.target().unwrap();
                let request: IdbOpenDbRequest = target.dyn_into().unwrap();
                let result = request.result().unwrap_or(JsValue::UNDEFINED);
                let _ = resolve.call1(&JsValue::NULL, &result);
            }) as Box<dyn FnMut(_)>);
            request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
            success.forget();

            let error = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("idb open failed"));
            }) as Box<dyn FnMut(_)>);
            request.set_onerror(Some(error.as_ref().unchecked_ref()));
            error.forget();
        })
    }

    fn transaction_promise(tx: &IdbTransaction) -> Promise {
        let tx = tx.clone();
        Promise::new(&mut |resolve, reject| {
            let resolve = resolve.clone();
            let reject = reject.clone();
            let complete = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let _ = resolve.call0(&JsValue::NULL);
            }) as Box<dyn FnMut(_)>);
            tx.set_oncomplete(Some(complete.as_ref().unchecked_ref()));
            complete.forget();

            let error = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("idb transaction failed"));
            }) as Box<dyn FnMut(_)>);
            tx.set_onerror(Some(error.as_ref().unchecked_ref()));
            error.forget();
        })
    }

    async fn open_db() -> Result<IdbDatabase, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let idb_factory = window
            .indexed_db()?
            .ok_or_else(|| JsValue::from_str("indexedDB unavailable"))?;

        let open_request: IdbOpenDbRequest = idb_factory.open_with_u32(DB_NAME, DB_VERSION)?;

        let onupgradeneeded = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().expect("upgrade event target");
            let request: IdbOpenDbRequest = target.dyn_into().expect("upgrade request");
            let db: IdbDatabase = request.result().unwrap().dyn_into().expect("upgrade db");
            if !object_store_exists(&db) {
                let _ = db.create_object_store(STORE_NAME);
            }
        }) as Box<dyn FnMut(_)>);
        open_request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));
        onupgradeneeded.forget();

        let db_value = JsFuture::from(open_request_promise(&open_request)).await?;
        db_value.dyn_into()
    }

    pub async fn put_string(key: &str, value: &str) -> Result<(), JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_NAME)?;
        let request = store.put_with_key(&JsValue::from_str(key), &JsValue::from_str(value))?;
        JsFuture::from(request_promise(&request)).await?;
        JsFuture::from(transaction_promise(&tx)).await?;
        Ok(())
    }

    pub async fn get_string(key: &str) -> Result<Option<String>, JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readonly)?;
        let store = tx.object_store(STORE_NAME)?;
        let request = store.get(&JsValue::from_str(key))?;
        let value = JsFuture::from(request_promise(&request)).await?;
        JsFuture::from(transaction_promise(&tx)).await?;
        if value.is_undefined() || value.is_null() {
            Ok(None)
        } else {
            Ok(value.as_string())
        }
    }

    pub async fn delete_string(key: &str) -> Result<(), JsValue> {
        let db = open_db().await?;
        let tx = db.transaction_with_str_and_mode(STORE_NAME, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(STORE_NAME)?;
        let request = store.delete(&JsValue::from_str(key))?;
        JsFuture::from(request_promise(&request)).await?;
        JsFuture::from(transaction_promise(&tx)).await?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{delete_string, get_string, put_string};

#[cfg(not(target_arch = "wasm32"))]
pub async fn put_string(_key: &str, _value: &str) -> Result<(), JsValue> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn get_string(_key: &str) -> Result<Option<String>, JsValue> {
    Ok(None)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_string(_key: &str) -> Result<(), JsValue> {
    Ok(())
}
