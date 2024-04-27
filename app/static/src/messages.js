export async function send_message(message, chat_id){
    const resp = await fetch("/database/send_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message: message, chat_id: chat_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}


export async function update_message(message, message_id){
    const resp = await fetch("/database/update_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message: message, message_id: message_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}

export async function view_message(message_id){
    const resp = await fetch("/database/view_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message_id: message_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}

export async function set_message_pinned(message_id, pinned){
    const resp = await fetch("/database/set_message_pinned", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message_id: message_id, pinned: pinned})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
}

export async function get_messages(chat_id, previous, limit){
    const resp = await fetch("/database/get_messages", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({chat_id: chat_id, previous: previous, limit:limit})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return {}
    }
    return await resp.json();
}

export async function get_message(message_id){
    const resp = await fetch("/database/get_message/"+message_id, {
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

export async function delete_message(message_id){
    const resp = await fetch("/database/delete_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message_id: message_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return "Error"
    }
    return "Sucsess"
}