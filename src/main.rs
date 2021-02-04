use carapax::{
    handler,
    longpoll::LongPoll,
    methods::{DeleteMessage, KickChatMember, UnbanChatMember},
    types::{Integer, Message, MessageData},
    Api, Config, Dispatcher, ExecuteError,
};
use dotenv::dotenv;
use std::env;

struct Context {
    api: Api,
}

fn get_new_chat_members(message: &Message) -> Option<Vec<Integer>> {
    if let MessageData::NewChatMembers(ref user_ids) = message.data {
        Some(user_ids.iter().map(|x| x.id).collect())
    } else {
        None
    }
}

#[handler]
async fn message_handler(context: &Context, message: Message) -> Result<(), ExecuteError> {
    let chat_id = message.get_chat_id();
    if let Some(user_ids) = get_new_chat_members(&message) {
        for user_id in user_ids {
            if let Err(err) = context.api.execute(KickChatMember::new(chat_id, user_id)).await {
                log::error!(
                    "Failed to kick chat member: {} (chat_id={} user_id={})",
                    err,
                    chat_id,
                    user_id
                )
            }
            if let Err(err) = context.api.execute(UnbanChatMember::new(chat_id, user_id)).await {
                log::error!(
                    "Failed to unban chat member: {} (chat_id={} user_id={})",
                    err,
                    chat_id,
                    user_id
                )
            }
        }
        if let Err(err) = context.api.execute(DeleteMessage::new(chat_id, message.id)).await {
            log::error!(
                "Failed to delete message: {} (chat_id={} message_id={})",
                err,
                chat_id,
                message.id
            )
        }
    } else if let MessageData::LeftChatMember(_) = message.data {
        context.api.execute(DeleteMessage::new(chat_id, message.id)).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let token = env::var("ONLY_COMMENTS_TOKEN").expect("ONLY_COMMENTS_TOKEN env var is not set or invalid");
    let config = Config::new(token);
    let api = Api::new(config).unwrap();
    let mut dispatcher = Dispatcher::new(Context { api: api.clone() });
    dispatcher.add_handler(message_handler);
    LongPoll::new(api, dispatcher).run().await;
}
