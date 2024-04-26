export async function add_contact(contact_id) {
    const resp = await fetch("/database/add_contact", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({contact_id: contact_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}

export async function find_user_email(email){
    const resp = await fetch("/database/find_user_email", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: email
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.json();
}

export async function find_user_username(email){
    const resp = await fetch("/database/find_user_username", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: email
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.json();
}

export async function find_user_phone(email){
    const resp = await fetch("/database/find_user_phone", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: email
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.json();
}

export async function delete_contact(contact_id){
    const resp = await fetch("/database/delete_contact", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({contact_id: contact_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}