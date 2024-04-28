export async function create_user(user) {
    let resp = await fetch('/database/create_user', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(user)
    });

    if (!resp.ok) {
        throw resp;
    }

    return await resp.json();
}

export async function update_user(user) {
    let resp = await fetch('/database/update_user', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(user)
    });

    if (!resp.ok) {
        throw resp;
    }

    return await resp.json();
}


export async function who_am_i(){
    const resp = await fetch("/database/who_am_i", {
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

export async function login(email, password){
    const resp = await fetch("/database/login", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({email: email, password: password})
    });

    if (!resp.ok) {
        throw resp;
    }
    return await resp.json();
}

export async function logout(){
    const resp = await fetch("/database/logout", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "POST",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        throw resp;
    }else{
        window.location.href = 'login.html'; // Redirect to login page
    }
}

export async function delete_account(){
    const resp = await fetch("/database/delete_account", {
        credentials: "same-origin",
        mode: "same-origin",
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        throw resp;
    }

    window.location.href = 'login.html'; // Redirect to login page
}

export async function get_username(user_id){
    const resp = await fetch("/database/get_username/"+user_id, {
        credentials: "same-origin",
        mode: "same-origin",
        method: "GET",
        headers: { "Content-Type": "application/json" },
    });

    if (!resp.ok) {
        throw resp;
    }

    return await resp.text();
}

export async function get_user(user_id){
    const resp = await fetch("/database/get_user/"+user_id, {
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