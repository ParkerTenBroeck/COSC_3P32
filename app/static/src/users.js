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


export { create_user, delete_account, login, logout, who_am_i };