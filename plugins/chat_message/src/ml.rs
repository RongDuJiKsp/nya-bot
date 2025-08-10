use crate::config::ChatConfig;
use anyhow::anyhow;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
};
use kovi::log::{error, warn};

async fn build_client() -> Client<OpenAIConfig> {
    let cfg = &ChatConfig::get().model;
    Client::with_config(
        OpenAIConfig::default()
            .with_api_base(&cfg.endpoint)
            .with_api_key(&cfg.key),
    )
}
async fn completion_chat(
    msg: Vec<ChatCompletionRequestMessage>,
    model: &str,
) -> Result<String, anyhow::Error> {
    let c = build_client().await;
    let res = c
        .chat()
        .create(
            CreateChatCompletionRequestArgs::default()
                .model(model)
                .max_tokens(ChatConfig::get().model.max_tokens)
                .messages(msg)
                .build()?,
        )
        .await?;
    res.choices
        .first()
        .and_then(|c| c.finish_reason)
        .and_then(|s| Some(warn!("model finished with {:?}", s)));
    if res.choices.is_empty() {
        error!("Model Null Output");
    }
    Ok(res
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .ok_or(anyhow!("Models No Response.Origin Output:{:?}", res))?)
}
async fn single_chat(s: &str, model: &str) -> Result<String, anyhow::Error> {
    completion_chat(
        vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage::from(s),
        )],
        model,
    )
    .await
}

pub async fn get_reply_as_nya_cat(
    chat_msg: Vec<ChatCompletionRequestMessage>,
) -> Result<String, anyhow::Error> {
    completion_chat(chat_msg, &ChatConfig::get().model.role_model).await
}
pub async fn get_reply_as_smart_nya_cat(q: &str) -> Result<String, anyhow::Error> {
    let prompt = &ChatConfig::get().model.smart_prompt;
    single_chat(
        &format!("{prompt}\n{q}"),
        &ChatConfig::get().model.smart_model,
    )
    .await
}
