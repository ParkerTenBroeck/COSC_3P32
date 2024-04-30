

async function create_dm(to){
    const resp = await fetch("/database/create_dm/" + to, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}



async function create_group(name){
    const resp = await fetch("/database/create_group", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({name: name})
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

async function create_channel(name){
    const resp = await fetch("/database/create_channel", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({name: name})
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

async function list_chats(){
    const resp = await fetch("/database/list_chats", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
    return await resp.json();
}

async function list_chat_members(chat_id){
    const resp = await fetch("/database/list_chat_members/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
    return await resp.json();
}

async function update_chat_notifications(chat_id, notifications){
    const resp = await fetch("/database/update_chat_notifications/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(notifications)
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}

async function mark_chat_read(chat_id){
    const resp = await fetch("/database/mark_chat_read/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST"
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}


async function leave_chat(chat_id){
    const resp = await fetch("/database/leave_chat/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST"
    });

    if (!resp.ok) {
        throw resp;
    }
}

async function join_chat(chat_id){
    const resp = await fetch("/database/join_chat/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST"
    });

    if (!resp.ok) {
        throw resp;
    }
}

export async function update_user_perm(chat_id, user_id, perm){
    const resp = await fetch("/database/update_chat_member_perm/", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id, user_id: user_id, perm: perm})
    });

    if (!resp.ok) {
        throw resp;
    }
}

async function delete_chat(chat_id){
    const resp = await fetch("/database/delete_chat/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE"
    });

    if (!resp.ok) {
        throw resp;
    }
}

export { create_channel, create_dm, create_group, delete_chat, join_chat, leave_chat, list_chat_members, list_chats, mark_chat_read, update_chat_notifications };