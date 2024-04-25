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

    return "Created";
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

async function private_message(message, to){
    const resp = await fetch("/database/private_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({message: message, to: to})
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
    return await resp.json();
}

async function update_message(message, mid){
    const resp = await fetch("/database/update_message", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        console.log("Status: " + resp.status)
        return Promise.reject("server")
    }
}


async function test(){
    await create_user({name: "ivy", email: "ivy", location: "ivy", password: "ivy", phone_number: "123", username: "ivy", location: "ivy"});
    await create_user({name: "parker", email: "parker", location: "parker", password: "parker", phone_number: "1233", username: "parker", location: "parker"});
    await login("parker", "parker");
    await private_message("hello!", 1);
}