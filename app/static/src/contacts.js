async function add_contact(user_id, contact_id) {
    const resp = await fetch("/database/add_contact", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({user_id: user_id, contact_id: contact_id})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}

export { add_contact };