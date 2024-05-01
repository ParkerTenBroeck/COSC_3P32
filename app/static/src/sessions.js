import * as messages from "./messages.js";
import * as chats from "./chats.js";
import * as users from "./users.js";



export class Session{

    
    user;

    user_cache = new Map();
    message_cache = new Map();
    chat_cache = new Map();

    chat_event_listener = [];
    self_event_listener = [];
    other_event_listener = [];

    constructor(){

    }

    async getChats(){
        let chats = await api.chats.list_chats()
        for (let chat of chats) {
            if (chat.seconary != null) {
                let other_id = chat.owner == this.user.user_id ? chat.seconary : chat.owner;
                chat.display_name = (await this.getUser(other_id)).display_name;
                chat.kind = "dm";
            } else if (chat.send_priv == 0) {
                chat.display_name= chat.name;
                chat.kind = "group";
            } else {
                chat.display_name = chat.name;
                chat.kind = "channel";
            }
            this.chat_cache.set(chat.chat_id, chat);
        }
        return chats;
    }

    async getChat(chat_id){
        if(this.chat_cache.get(chat_id) == null){
            let chat = api.chats.get_chat(chat_id);

            if (chat.seconary != null) {
                let other_id = chat.owner == this.user.user_id ? chat.seconary : chat.owner;
                chat.display_name = (await this.getUser(other_id)).display_name;
                chat.kind = "dm";
            } else if (chat.send_priv == 0) {
                chat.display_name= chat.name;
                chat.kind = "group";
            } else {
                chat.display_name = chat.name;
                chat.kind = "channel";
            }
            this.chat_cache.set(
                chat_id,
                chat
            );
        }
        return await this.chat_cache.get(chat_id);
    }

    async getUser(user_id){
        if(user_id == null){
            return {
                user_id: null,
                display_name: "<deleted>",
                bio: "<deleted>",
                pfp_file_id: null,
                pfp: "https://upload.wikimedia.org/wikipedia/commons/thumb/2/2c/Default_pfp.svg/2048px-Default_pfp.svg.png"
            }
        }
        if(this.user_cache.get(user_id) == null){
            try{
                this.user_cache.set(
                    user_id,
                    api.users.get_user(user_id)
                );
            }catch(e){
                this.user_cache.set(
                    user_id,
                    (async () => {
                        return {
                            user_id: null,
                            display_name: "<deleted>",
                            bio: "<deleted>",
                            pfp_file_id: null
                        }
                    })()
                )
            }
        }
        try{
            let user = await this.user_cache.get(user_id);

            if (user.pfp_file_id == null) {
                user.pfp = "https://upload.wikimedia.org/wikipedia/commons/thumb/2/2c/Default_pfp.svg/2048px-Default_pfp.svg.png"
            } else {
                user.pfp = "/database/attachments/" + user.pfp_file_id;
            }
            return user;
        }catch(e){
            throw e;
        }
    }
    async getMessage(message_id){
        if(this.message_cache.get(message_id) == null){
            
            let message = await api.messages.get_message(message_id);
            if (message.attachment_id != null){
                message.attachment = "/database/attachments/" + message.attachment_id;
            }else{
                message.attachment = null;
            }

            this.message_cache.set(
                message_id,
                (async () => message)()
            );
        }
        return await this.message_cache.get(message_id);
    }

    async getMessages(chat_id, previous, limit){
        let list = await api.messages.get_messages(chat_id, previous, limit);
        for (const message of list){
            if (message.attachment_id != null){
                message.attachment = "/database/attachments/" + message.attachment_id;
            }else{
                message.attachment = null;
            }

            this.message_cache.set(
                message.message_id,
                (async () => message)()
            );
        }
        return list;
    }

    addSelfListener(listener){
        this.self_event_listener.push(listener)
    }
    addChatListener(listener){
        this.chat_event_listener.push(listener)
    }
    addOtherListener(listener){
        this.other_event_listener.push(listener)
    }

    async begin(){
        this.event = new EventSource("/database/open_session")
        
        this.user = await users.who_am_i();
        this.event.onmessage = async (e) => {
            const event = JSON.parse(e.data);
            switch (event.tag){
                case "WhoAmI":
                    this.user = await users.who_am_i();
                    this.self_event_listener.forEach((e) => e(event));
                    break;

                case "UserDeleted":
                    this.user_cache.delete(event.user);
                    if(event.user != this.user.user_id){
                        this.other_event_listener.forEach((e) => e(event));
                    }else{
                        this.self_event_listener.forEach((e) => e(event));
                        this.user = await users.who_am_i();
                    }
                    break;
                case "UserUpdated":
                    this.user_cache.delete(event.user);
                    if(event.user != this.user.user_id){
                        this.chat_event_listener.forEach((e) => e(event));
                    }else{
                        this.user = await users.who_am_i();
                        this.self_event_listener.forEach((e) => e(event));
                    }
                    break;

                case "UserJoined":
                    if(event.user != this.user.user_id){
                        this.chat_event_listener.forEach((e) => e(event));
                    }else{
                        this.self_event_listener.forEach((e) => e(event));
                    }
                    break;
                case "UserLeft":
                    if(event.user != this.user.user_id){
                        this.chat_event_listener.forEach((e) => e(event));
                    }else{
                        this.self_event_listener.forEach((e) => e(event));
                    }
                    break;
                    
                case "ChatUpdated":
                    this.chat_cache.delete(event.chat);
                    this.chat_event_listener.forEach((e) => e(event));
                    break;
                case "NewMessage":
                    this.chat_event_listener.forEach((e) => e(event));
                    break;
                case "MessageEdited":
                    this.message_cache.delete(event.message);
                    this.chat_event_listener.forEach((e) => e(event));
                    break;
                case "MessageDeleted":
                    this.message_cache.delete(event.message);
                    this.chat_event_listener.forEach((e) => e(event));
                    break;
            }
            if(this.user == null){
                window.location.href = 'login.html';
            }
        }
    }

    async subscribe_chat(chat_id){
        const resp = await fetch("/database/session_subscribe_to_chat/"+chat_id, {
            credentials: "same-origin",
            mode: "same-origin",
            method: "POST",
        });
    
        if (!resp.ok) {
            throw resp;
        }
    }

    async unsubscribe_chat(){
        const resp = await fetch("/database/session_unsubscribe_chat", {
            credentials: "same-origin",
            mode: "same-origin",
            method: "POST",
        });
    
        if (!resp.ok) {
            throw resp;
        }
    }

    async subscribe_user(user_id){
        const resp = await fetch("/database/session_subscribe_to_user/"+user_id, {
            credentials: "same-origin",
            mode: "same-origin",
            method: "POST",
        });
    
        if (!resp.ok) {
            throw resp;
        }
    }

    async unsubscribe_user(user_id){
        const resp = await fetch("/database/session_unsubscribe_from_user/"+user_id, {
            credentials: "same-origin",
            mode: "same-origin",
            method: "POST",
        });
    
        if (!resp.ok) {
            throw resp;
        }
    }
}