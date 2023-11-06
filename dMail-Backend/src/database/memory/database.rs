// use chashmap::CHashMap;
// use once_cell::sync::Lazy;
// use std::sync::atomic::AtomicU64;
// use std::sync::Mutex;

// use crate::chat::*;
// use crate::config::datatype::{ChatID, UserID};
// pub struct UserBaseData {
//     pub user_name: String,
//     pub password: String,
//     pub email: String,
// }

// // 不需要长时间读，或许使用Mutex更好一些
// // TODO : 比较spin、parking_lot等第三方锁实现的性能
// #[allow(non_upper_case_globals)]
// pub static user_array: Lazy<Mutex<Vec<UserBaseData>>> = Lazy::new(|| Mutex::new(vec![]));
// #[allow(non_upper_case_globals)]
// pub static user_email_hash_map: Lazy<CHashMap<String, UserID>> = Lazy::new(|| CHashMap::new());
// #[allow(non_upper_case_globals)]
// pub static chat_array: Lazy<Mutex<Vec<Chat>>> = Lazy::new(|| Mutex::new(vec![]));
// #[allow(non_upper_case_globals)]
// pub static chat_message: Lazy<Mutex<Vec<Vec<ChatMessage>>>> = Lazy::new(|| Mutex::new(vec![]));
// #[allow(non_upper_case_globals)]
// pub static user_chats: Lazy<Mutex<Vec<Vec<ChatID>>>> = Lazy::new(|| Mutex::new(vec![]));
// #[allow(non_upper_case_globals)]
// pub static meesage_current_id: AtomicU64 = AtomicU64::new(0);
