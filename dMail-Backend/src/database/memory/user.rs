// use super::database::*;
// use crate::chat::*;
// use crate::config::datatype::{ChatID, MessageID, UserID};
// use crate::user::*;

// pub fn user_register(data: UserRegisterData) -> UserRegisterResponse {
//     if user_email_hash_map.contains_key(&data.email) {
//         return UserRegisterResponse::EmailRegistered;
//     }

//     let mut user_array_lock = match user_array.lock() {
//         Ok(lock) => lock,
//         Err(_) => return UserRegisterResponse::ServerError,
//     };

//     user_array_lock.push(UserBaseData {
//         user_name: data.user_name,
//         password: data.password,
//         email: data.email.clone(),
//     });

//     let user_id = (user_array_lock.len() - 1) as u32;
//     user_email_hash_map.insert(data.email, user_id);
//     user_chats.lock().unwrap().push(vec![]);

//     return UserRegisterResponse::Success { user_id: user_id };
// }

// pub fn get_user_id_by_email(email: &String) -> Option<UserID> {
//     return user_email_hash_map.get(email).map(|guard| *guard);
// }

// pub fn user_login(data: &UserLoginData) -> UserLoginResponse {
//     let user_array_lock = match user_array.lock() {
//         Ok(lock) => lock,
//         Err(_) => return UserLoginResponse::ServerError,
//     };

//     let user_id_opt = get_user_id_by_email(&data.email);

//     if user_id_opt.is_none() {
//         return UserLoginResponse::UserNotFound;
//     }

//     let user_id = user_id_opt.unwrap();

//     match user_array_lock.get(user_id as usize) {
//         Some(user_data) => {
//             if user_data.password != *data.password.as_ref().unwrap() {
//                 return UserLoginResponse::PasswordError;
//             }
//         }
//         None => return UserLoginResponse::UserNotFound,
//     }

//     return UserLoginResponse::Success { user_id: user_id };
// }

// pub fn check_user_can_send_in_chat(
//     user_id: UserID,
//     chat_id: ChatID,
// ) -> Result<ChatType, UserSendMessageResponseState> {
//     let chat_array_lock = match chat_array.lock() {
//         Ok(lock) => lock,
//         Err(_) => return Err(UserSendMessageResponseState::ServerError),
//     };

//     let members = match chat_array_lock.get(chat_id as usize) {
//         Some(chat) => &chat.members,
//         None => return Err(UserSendMessageResponseState::ChatNotFound),
//     };

//     match members {
//         ChatMembers::Private((id1, id2)) => {
//             if *id1 == user_id || *id2 == user_id {
//                 return Ok(ChatType::Private((*id1, *id2)));
//             } else {
//                 return Err(UserSendMessageResponseState::UserNotInChat);
//             }
//         }
//         ChatMembers::Group(ids) => {
//             if ids.contains(&user_id) {
//                 return Ok(ChatType::Group(ids.len() as UserID));
//             } else {
//                 return Err(UserSendMessageResponseState::UserNotInChat);
//             }
//         }
//     }

//     // return Ok(ChatType::Private);
// }

// pub fn write_message_to_chat(
//     _send_msg_data: UserSendMessageData,
//     _sender_id: UserID,
// ) -> Result<(ChatMessage, MessageID), ()> {
//     todo!()
//     // let mut chat_msg_array_lock = match chat_message.lock() {
//     //     Ok(chat_msg) => chat_msg,
//     //     Err(_) => return Err(()),
//     // };

//     // let msg = ChatMessage {
//     //     chat_id: send_msg_data.chat_id,
//     //     sender_id: sender_id,
//     //     in_chat_id: chat_msg_array_lock[send_msg_data.chat_id as usize].len() as u64,
//     //     text: send_msg_data.text,
//     //     timestamp: send_msg_data.timestamp,
//     // };

//     // chat_msg_array_lock[msg.chat_id as usize].push(msg.clone());

//     // return Ok((msg, send_msg_data.client_id));
// }

// pub fn get_chat_user_list(chat_id: ChatID) -> Result<ChatMembers, ()> {
//     let chat_lock = if let Ok(lock) = chat_array.lock() {
//         lock
//     } else {
//         return Err(());
//     };

//     match chat_lock.get(chat_id as usize) {
//         Some(chat) => {
//             // 这里的克隆是因为在远端数据库接口传回来的不可避免的要再申请内存
//             // 内存这边其实也就无所谓了（），如果内存单独传引用接口还得改
//             return Ok(chat.members.clone());
//         }
//         None => {
//             return Err(());
//         }
//     }
// }

// pub fn create_group_chat(creator: Option<UserID>) -> Result<ChatID, ()> {
//     let mut chat_lock = if let Ok(lock) = chat_array.lock() {
//         lock
//     } else {
//         return Err(());
//     };

//     let chat_id = chat_lock.len() as ChatID;

//     chat_lock.push(Chat {
//         id: chat_id,
//         members: if let Some(user_id) = creator {
//             ChatMembers::Group(vec![user_id])
//         } else {
//             ChatMembers::Group(vec![])
//         },
//     });

//     {
//         let mut chat_msg_array_lock = match chat_message.lock() {
//             Ok(chat_msg) => chat_msg,
//             Err(_) => return Err(()),
//         };

//         chat_msg_array_lock.push(vec![])
//     }

//     if let Some(user_id) = creator {
//         let mut user_chats_lock = user_chats.lock().unwrap();

//         let chat_list = user_chats_lock.get_mut(user_id as usize).unwrap();

//         chat_list.push(chat_id);
//     }

//     return Ok(chat_id);
// }

// pub fn add_user_to_group_chat(chat_id: ChatID, user_id: UserID) -> Result<(), ()> {
//     let mut chat_lock = if let Ok(lock) = chat_array.lock() {
//         lock
//     } else {
//         return Err(());
//     };

//     let chat = match chat_lock.get_mut(chat_id as usize) {
//         Some(chat) => chat,
//         None => return Err(()),
//     };

//     match &mut chat.members {
//         ChatMembers::Private(_) => return Err(()),
//         ChatMembers::Group(ids) => {
//             ids.push(user_id);
//             let mut user_chats_lock = user_chats.lock().unwrap();

//             let chat_list = user_chats_lock.get_mut(user_id as usize).unwrap();
//             chat_list.push(chat_id);
//         }
//     }

//     return Ok(());
// }

// // pub fn get_user_chat_list_with_info(
// //     user_id: UserID,
// //     start_id: ChatID,
// // ) -> Result<Vec<ChatInfo>, ()> {
// //     let mut user_chats_lock = user_chats.lock().unwrap();

// //     let chat_list = match user_chats_lock.get(user_id as usize) {
// //         Some(list) => list,
// //         None => return Err(()),
// //     };

// // }

// pub fn get_user_info(user_id: UserID) -> GetUserInfoResponse {
//     let users_lock = if let Ok(lock) = user_array.lock() {
//         lock
//     } else {
//         return GetUserInfoResponse::ServerError;
//     };

//     if let Some(data) = users_lock.get(user_id as usize) {
//         return GetUserInfoResponse::Success(UserInfo {
//             user_name: data.email.clone(),
//             user_id: 0,
//             avater_hash: todo!(),
//         });
//     } else {
//         return GetUserInfoResponse::UserNotFound;
//     }
// }
