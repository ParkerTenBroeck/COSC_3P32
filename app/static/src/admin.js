async function list_users(){
    const resp = await fetch("/database/list_users", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

export { list_users };