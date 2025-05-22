use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleEmojiPicker,
    AddEmoji(String),
    ReactToMessage(usize, String),
    SetTheme(Theme), 
}

#[derive(Deserialize, Serialize, Clone)]
struct MessageData {
    from: String,
    message: String,
    #[serde(default)]
    reactions: Vec<(String, String)>, 
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
    Reaction,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Classic,
    Midnight,
    Sky,
    Forest,
}
impl Theme {
    fn bg_class(&self) -> &'static str {
        match self {
            Theme::Classic => "bg-gray-100",
            Theme::Midnight => "bg-gray-900",
            Theme::Sky => "bg-blue-100",
            Theme::Forest => "bg-green-100",
        }
    }
    fn msg_class(&self) -> &'static str {
        match self {
            Theme::Classic => "bg-white text-gray-800",
            Theme::Midnight => "bg-gray-800 text-gray-100",
            Theme::Sky => "bg-blue-50 text-blue-900",
            Theme::Forest => "bg-green-50 text-green-900",
        }
    }
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    emoji_picker_open: bool,
    selected_emoji: Option<String>,
    theme: Theme,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            emoji_picker_open: false,
            selected_emoji: None,
            theme: Theme::Classic,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    MsgTypes::Reaction => {
                        // data: { messageIndex, emoji, from }
                        if let Some(data) = msg.data {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                                let idx = val["messageIndex"].as_u64().unwrap_or(0) as usize;
                                let emoji = val["emoji"].as_str().unwrap_or("").to_string();
                                let from = val["from"].as_str().unwrap_or("").to_string();
                                if let Some(msg) = self.messages.get_mut(idx) {
                                    msg.reactions.retain(|(u, _)| *u != from);
                                    if !emoji.is_empty() {
                                        msg.reactions.push((from, emoji));
                                    }
                                }
                                return true;
                            }
                        }
                        false
                    }
                    _ => false,
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
            Msg::ToggleEmojiPicker => {
                self.emoji_picker_open = !self.emoji_picker_open;
                true
            }
            Msg::AddEmoji(emoji) => {
                self.selected_emoji = Some(emoji.clone());
                if let Some(input) = self.chat_input.cast::<HtmlInputElement>() {
                    let mut val = input.value();
                    val.push_str(&emoji);
                    input.set_value(&val);
                }
                self.emoji_picker_open = false;
                true
            }
            Msg::ReactToMessage(idx, emoji) => {
                let (user, _) = ctx.link().context::<User>(Callback::noop()).expect("context to be set");
                let username = user.username.borrow().clone();
                if let Some(msg) = self.messages.get(idx) {
                    // Find current reaction (if any)
                    let current = msg.reactions.iter().find(|(u, _)| *u == username).map(|(_, e)| e.clone());
                    let new_emoji = if let Some(cur) = current {
                        if cur == emoji { "".to_string() } else { emoji }
                    } else {
                        emoji
                    };
                    let reaction_msg = WebSocketMessage {
                        message_type: MsgTypes::Reaction,
                        data: Some(serde_json::to_string(&(idx, new_emoji)).unwrap()),
                        data_array: None,
                    };
                    let _ = self.wss.tx.clone().try_send(serde_json::to_string(&reaction_msg).unwrap());
                }
                false
            }
            Msg::SetTheme(theme) => {
                self.theme = theme;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle_emoji = ctx.link().callback(|_| Msg::ToggleEmojiPicker);

        // Theme switcher buttons
        let set_theme = |theme: Theme| ctx.link().callback(move |_| Msg::SetTheme(theme.clone()));
        let theme = &self.theme;

        let (user, _) = ctx.link().context::<User>(Callback::noop()).expect("context to be set");
        let username = user.username.borrow().clone();

        html! {
            <div class={format!("flex w-screen h-screen {}", theme.bg_class())}>
                <div class="flex-none w-56 h-screen bg-gray-100">
                    <div class="p-3 flex justify-between items-center">
                        <div class="text-xl">{ "Users" }</div>
                    </div>
                    <div class="user-counter">
                        { format!("Online users: {}", self.users.len()) }
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2">
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                    <div class="mt-6 px-3">
                        <div class="text-xs text-gray-500 mb-1">{ "Theme:" }</div>
                        <div class="flex flex-wrap gap-2">
                            <button onclick={set_theme(Theme::Classic)} class={format!("px-2 py-1 rounded {}", if *theme == Theme::Classic { "bg-blue-500 text-white" } else { "bg-white text-gray-800 border" })}>{ "Classic" }</button>
                            <button onclick={set_theme(Theme::Midnight)} class={format!("px-2 py-1 rounded {}", if *theme == Theme::Midnight { "bg-blue-900 text-white" } else { "bg-white text-gray-800 border" })}>{ "Midnight" }</button>
                            <button onclick={set_theme(Theme::Sky)} class={format!("px-2 py-1 rounded {}", if *theme == Theme::Sky { "bg-blue-300 text-blue-900" } else { "bg-white text-gray-800 border" })}>{ "Sky" }</button>
                            <button onclick={set_theme(Theme::Forest)} class={format!("px-2 py-1 rounded {}", if *theme == Theme::Forest { "bg-green-700 text-white" } else { "bg-white text-gray-800 border" })}>{ "Forest" }</button>
                        </div>
                    </div>
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300 flex items-center px-4">
                        <div class="text-xl font-bold">{ "💬 Chat" }</div>
                    </div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-300 p-4">
                        {
                            self.messages.iter().enumerate().map(|(idx, m)| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                let avatar = user.map(|u| u.avatar.clone()).unwrap_or_default();
                                let user_reaction = m.reactions.iter().find(|(u, _)| *u == username).map(|(_, e)| e.clone());
                                html!{
                                    <div class={format!("flex flex-col items-start w-3/6 {} m-8 rounded-tl-lg rounded-tr-lg rounded-br-lg shadow", theme.msg_class())}>
                                        <div class="flex items-end">
                                            <img class="w-8 h-8 rounded-full m-3" src={avatar} alt="avatar"/>
                                            <div class="p-3">
                                                <div class="text-sm font-semibold">
                                                    {m.from.clone()}
                                                </div>
                                                <div class="text-xs">
                                                    if m.message.ends_with(".gif") {
                                                        <img class="mt-3" src={m.message.clone()}/>
                                                    } else {
                                                        {m.message.clone()}
                                                    }
                                                </div>
                                            </div>
                                        </div>
                                        <div class="flex space-x-2 mt-2">
                                            { for ["👍", "😂", "❤️"].iter().map(|emoji| {
                                                let emoji = emoji.to_string();
                                                let selected = user_reaction.as_deref() == Some(&emoji);
                                                let cb = ctx.link().callback({
                                                    let emoji = emoji.clone();
                                                    move |_| Msg::ReactToMessage(idx, emoji.clone())
                                                });
                                                html! {
                                                    <button
                                                        onclick={cb}
                                                        class={if selected { "font-bold border border-blue-500 bg-blue-100" } else { "" }}
                                                    >{ emoji.clone() }</button>
                                                }
                                            })}
                                        </div>
                                        <div class="flex flex-wrap text-xs mt-1">
                                            { for m.reactions.iter().map(|(user, emoji)| html!{ <span class="mr-2">{ format!("{} {}", emoji, user) }</span> }) }
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-14 flex px-3 items-center relative">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class={format!("block w-full py-2 pl-4 mx-3 rounded-full outline-none focus:text-gray-700 {}", theme.bg_class())} name="message" required=true />
                        <button onclick={toggle_emoji} class="mx-1">{ "😀" }</button>
                        { if self.emoji_picker_open {
                            html! {
                                <div class="emoji-picker bg-white border rounded p-2 absolute z-10 right-16 bottom-16">
                                    { for ["😀","😂","😊","🥰","😍","😎","🙄","😴","🤔","🤯","😱","🥳","😭","😡","🤢","👍","👎","👏","🙏","💪","🤝","❤️","💔","💯","🔥","💩","🎉","✨","🌈","⭐","🎁","🏆"].iter().map(|emoji| {
                                        let emoji_str = emoji.to_string();
                                        let cb = ctx.link().callback({
                                            let emoji_str = emoji_str.clone();
                                            move |_| Msg::AddEmoji(emoji_str.clone())
                                        });
                                        html! {
                                            <button onclick={cb} class="text-2xl hover:bg-gray-100 rounded p-1">{ emoji }</button>
                                        }
                                    })}
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center color-white">
                            <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}