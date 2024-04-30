export async function send_message(message, chat_id, attachment_id, reply){
    const resp = await fetch("/database/send_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message: message, chat_id: chat_id, attachment_id: attachment_id, reply: reply})
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}


export async function update_message(message, message_id){
    const resp = await fetch("/database/update_message/" + message_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message: message})
    });

    if (!resp.ok) {
        throw resp;
    }
}

export async function view_message(message_id){
    const resp = await fetch("/database/view_message/"+message_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        throw resp;
    }
}

export async function set_message_pinned(message_id, pinned){
    const resp = await fetch("/database/set_message_pinned/" + message_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(pinned)
    });

    if (!resp.ok) {
        throw resp;
    }
}

export async function get_messages(chat_id, previous, limit){
    const resp = await fetch("/database/get_messages/" + chat_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({previous: previous, limit:limit})
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

export async function get_message(message_id){
    const resp = await fetch("/database/get_message/"+message_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET"
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

export async function delete_message(message_id){
    const resp = await fetch("/database/delete_message/" + message_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE"
    });

    if (!resp.ok) {
        throw resp;
    }
}