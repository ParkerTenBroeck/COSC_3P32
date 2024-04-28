

export async function upload_file(name, contents){
    const resp = await fetch("/database/upload_file/" + name, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: contents
    });

    if (!resp.ok) {
        throw resp;
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
        throw resp;
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
        throw resp;
    }

    return await resp.blob();
}