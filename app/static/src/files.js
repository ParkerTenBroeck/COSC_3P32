

export async function upload_file(name, contents){
    const resp = await fetch("/database/upload_file/" + name, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: contents
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.json();
}

export async function get_file_with_name(id, name){
    const resp = await fetch("/database/attachments/" + id + "/" + name, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: contents
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.blob();
}

export async function get_file(id){
    const resp = await fetch("/database/attachments/" + id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: contents
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    return await resp.blob();
}