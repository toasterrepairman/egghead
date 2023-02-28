mod images;

pub use images::*;
use {
    serenity::{
        framework::standard::CommandResult,
        model::prelude::{Message, MessageId},
        prelude::{Context, TypeMapKey},
    },
    std::collections::HashMap,
};

pub struct MessageLink;

impl TypeMapKey for MessageLink {
    type Value = HashMap<MessageId, Message>;
}

pub async fn delete_if_linked(ctx: &Context, msg: &MessageId) -> CommandResult {
    {
        //  delete the message if the message is linked
        let data = ctx.data.read().await;
        let links = data
            .get::<MessageLink>()
            .ok_or("Message link map hasn't been instantiated")?;

        let link = links
            .get(msg)
            .ok_or("Message did not have a link to embed")?;
        link.delete(&ctx.http).await?;
    }
    {
        //  remove the message from links if it was able to be deleted
        let mut data = ctx.data.write().await;
        let links = data
            .get_mut::<MessageLink>()
            .ok_or("Message link map hasn't been instantiated")?;
        links.remove(msg);
        Ok(())
    }
}

pub async fn link(ctx: &Context, from: MessageId, to: Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let links = data
        .get_mut::<MessageLink>()
        .ok_or("Message link map hasn't been instantiated")?;

    links.insert(from, to);
    Ok(())
}
