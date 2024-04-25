// async function create(title, text){
//     const dataToSend = JSON.stringify({"text": text, "title": title});
//     let dataReceived = ""; 
//     const resp = await fetch("/database/", {
//         credentials: "same-origin",
//         mode: "same-origin",
//         method: "post",
//         headers: { "Content-Type": "application/json" },
//         body: dataToSend
//     });
//     if (resp.status === 201) {
//         return await resp.json() 
//     } else {
//         console.log("Status: " + resp.status)
//         return Promise.reject("server")
//     }
// }

// async function get(id){
//     const resp = await fetch("/database/" + id, {
//         credentials: "same-origin",
//         mode: "same-origin",
//         method: "GET",
//         headers: { "Content-Type": "application/json" },
//     });

//     if (resp.status === 200) {
//         return await resp.json()
//     } else {
//         console.log("Status: " + resp.status)
//         return Promise.reject("server")
//     }
// }

// async function list(){
//     const resp = await fetch("/database/list_users", {
//         credentials: "same-origin",
//         mode: "same-origin",
//         method: "GET",
//         headers: { "Content-Type": "application/json" },
//     });

//     if (resp.status === 200) {
//         return await resp.json();
//     } else {
//         console.log("Status: " + resp.status)
//         return Promise.reject("server")
//     }
// }


async function create_user(user) {
    let resp = await fetch('/database/create_user', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(user)
    });

    if(resp.status == 409){
        return "Conflict"
    }

    if (!resp.ok) {
        throw new Error('Network response was not ok');
    }

    return await resp.json();
}

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

async function list_users(){
    const resp = await fetch("/database/list_users", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}

async function who_am_i(){
    const resp = await fetch("/database/who_am_i", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}

async function login(email, password){
    const resp = await fetch("/database/login", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({email: email, password: password})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}

async function logout(){
    const resp = await fetch("/database/logout", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }else{
        window.location.href = 'login.html'; // Redirect to login page
    }
}

async function delete_account(){
    const resp = await fetch("/database/delete_account", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }

    window.location.href = 'login.html'; // Redirect to login page
}



async function delete_message(message_id){
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

async function send_message(message, chat_id){
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

async function update_message(message, message_id){
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


async function test(){
    const uid1 = await create_user({name: "ivy", email: "ivy", location: "ivy", password: "ivy", phone_number: "123", username: "ivy", location: "ivy"});
    const uid2 = await create_user({name: "parker", email: "parker", location: "parker", password: "parker", phone_number: "1233", username: "parker", location: "parker"});
    await login("parker", "parker");
    let dm_id = await create_dm(uid1);
    await send_message("hello!", dm_id);

    await login("ivy", "ivy");
    await send_message("hiiii!", dm_id);
    const mid = await send_message("byr", dm_id);
    await update_message("bye", mid);

    await delete_chat(dm_id);

    await login("parker", "parker");
    const group_id = create_group("BRUH!");
    await send_message("asdlfjhasdlkfgjhsad", group_id);
}