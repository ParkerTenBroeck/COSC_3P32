

async function create_dm(to){
    const resp = await fetch("/database/create_dm", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({other: to})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
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
        console.log("Status: " + resp.status)
        return Promise.reject("server")
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
        console.log("Status: " + resp.status)
        return Promise.reject("server")
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
    const resp = await fetch("/database/list_chat_members", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
    return await resp.json();
}

async function update_chat_notifications(chat_id, notifications){
    const resp = await fetch("/database/update_chat_notifications", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id, notifications:notifications})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}

async function mark_chat_read(chat_id){
    const resp = await fetch("/database/mark_chat_read", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}


async function leave_chat(chat_id){
    const resp = await fetch("/database/leave_chat", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}

async function join_chat(chat_id){
    const resp = await fetch("/database/join_chat", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}

async function delete_chat(chat_id){
    const resp = await fetch("/database/delete_chat", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}

export { create_channel, create_dm, create_group, delete_chat, join_chat, leave_chat, list_chat_members, list_chats, mark_chat_read, update_chat_notifications };