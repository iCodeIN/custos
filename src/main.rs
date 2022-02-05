use carapax::{
    longpoll::LongPoll,
    methods::{BanChatMember, DeleteMessage, UnbanChatMember},
    types::{ChatId, Integer, Message, MessageData, UserId},
    Api, App, Context, ExecuteError, Ref,
};
use dotenv::dotenv;
use std::{env, error::Error, fmt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    let token = env::var("CUSTOS_TOKEN").expect("CUSTOS_TOKEN env var is not set or invalid");
    let api = Api::new(token).expect("Failed to create Api");
    let mut context = Context::default();
    context.insert(api.clone());
    LongPoll::new(api, App::new(context, handler)).run().await;
}

async fn handler(api: Ref<Api>, message: Message) -> Result<(), AppError> {
    let chat_id = message.get_chat_id();
    if let Some(user_ids) = get_new_chat_members(&message) {
        for user_id in user_ids {
            api.execute(BanChatMember::new(chat_id, user_id))
                .await
                .map_err(|err| AppError::ban(err, chat_id, user_id))?;
            api.execute(UnbanChatMember::new(chat_id, user_id))
                .await
                .map_err(|err| AppError::unban(err, chat_id, user_id))?;
        }
        api.execute(DeleteMessage::new(chat_id, message.id))
            .await
            .map_err(|err| AppError::delete_message(err, chat_id, message.id))?;
    } else if let MessageData::LeftChatMember(_) = message.data {
        api.execute(DeleteMessage::new(chat_id, message.id))
            .await
            .map_err(|err| AppError::delete_message(err, chat_id, message.id))?;
    }
    Ok(())
}

fn get_new_chat_members(message: &Message) -> Option<Vec<Integer>> {
    if let MessageData::NewChatMembers(ref user_ids) = message.data {
        Some(user_ids.iter().map(|x| x.id).collect())
    } else {
        None
    }
}

#[derive(Debug)]
enum AppError {
    Ban(ExecuteError, ChatId, UserId),
    Unban(ExecuteError, ChatId, UserId),
    DeleteMessage(ExecuteError, ChatId, Integer),
}

impl AppError {
    fn ban<C, U>(source: ExecuteError, chat_id: C, user_id: U) -> Self
    where
        C: Into<ChatId>,
        U: Into<UserId>,
    {
        Self::Ban(source, chat_id.into(), user_id.into())
    }

    fn unban<C, U>(source: ExecuteError, chat_id: C, user_id: U) -> Self
    where
        C: Into<ChatId>,
        U: Into<UserId>,
    {
        Self::Unban(source, chat_id.into(), user_id.into())
    }

    fn delete_message<C>(source: ExecuteError, chat_id: C, message_id: Integer) -> Self
    where
        C: Into<ChatId>,
    {
        Self::DeleteMessage(source, chat_id.into(), message_id)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AppError::*;
        match self {
            Ban(source, chat_id, user_id) => write!(
                out,
                "failed to ban chat member: {source} (chat_id={chat_id}, user_id={user_id})"
            ),
            Unban(source, chat_id, user_id) => write!(
                out,
                "failed to unban chat member: {source} (chat_id={chat_id}, user_id={user_id})"
            ),
            DeleteMessage(source, chat_id, message_id) => write!(
                out,
                "failed to delete message: {source} (chat_id={chat_id}, message_id={message_id})"
            ),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::AppError::*;
        Some(match self {
            Ban(source, _, _) => source,
            Unban(source, _, _) => source,
            DeleteMessage(source, _, _) => source,
        })
    }
}
