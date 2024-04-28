async function showSettings() {
  await reload_user_data();
  var chatArea = document.getElementById("chatArea");
  
  let pfp;
  if (userData.pfp_file_id == null) {
    pfp = "https://upload.wikimedia.org/wikipedia/commons/thumb/2/2c/Default_pfp.svg/2048px-Default_pfp.svg.png"
  } else {
    pfp = "/database/attachments/" + userData.pfp_file_id;
  }

  chatArea.innerHTML = `
        <div class="settings-menu">

        <form id="profileForm">
            <img src="${pfp}" alt="Profile Picture" class="pfp">
            <label for="phone_number">Phone Number:</label>
            <input type="tel" id="phone_number" name="phone_number" required><br><br>

            <label for="name">Name:</label>
            <input type="text" id="name" name="name" required><br><br>

            <label for="email">Email:</label>
            <input type="email" id="email" name="email" required><br><br>

            <label for="location">Location:</label>
            <input type="text" id="location" name="location" required><br><br>

            <label for="username">Username:</label>
            <input type="text" id="username" name="username" required><br><br>

            <label for="password">Password:</label>
            <input type="password" id="password" name="password" required><br><br>

            <label for="bio">Bio:</label><br>
            <textarea id="bio" name="bio" required></textarea><br><br>

            <div class="dropzone" id="fileDropzone">
                <input type="file" id="fileInput" accept="image/*">
                <p>Drag and drop your profile picture here, or click to select</p>
            </div>
            
            <button type="button" onclick="updateProfile()">Update</button>
          </form>
          <button onclick="api.users.logout()">Logout</button>
        </div>`;

        reloadProfile();

  const dropzone = document.getElementById('fileDropzone');
  const fileInput = document.getElementById('fileInput');

  dropzone.addEventListener('dragover', (event) => {
    event.preventDefault();
    dropzone.style.backgroundColor = '#f0f0f0';
  });

  dropzone.addEventListener('dragleave', () => {
    dropzone.style.backgroundColor = '';
  });

  dropzone.addEventListener('drop', (event) => {
    event.preventDefault();
    dropzone.style.backgroundColor = '';

    const file = event.dataTransfer.files[0];
    handleFile(file);
  });

  fileInput.addEventListener('change', (event) => {
    const file = event.target.files[0];
    handleFile(file);
  });

  async function handleFile(file) {
    let reader = new FileReader();
    reader.readAsBinaryString(file);
    
    const fid = await api.files.upload_file(file.name, file);
    await api.users.update_user_pfp(fid);
    await showSettings();
  }
}

async function reloadProfile(){

  for (key of Object.keys(userData)){
      const el = document.getElementById(key)
      if(el != null){
          el.value = userData[key]
      }
  }
}

async function updateProfile() {
  // Get form data
  const form = document.getElementById('profileForm');
  const formData = new FormData(form);
  
  var object = {};
    formData.forEach((value, key) => object[key] = value);  

  api.users.update_user(object);

  await reloadProfile();
}

async function createChat() {

  await reload_chat_data();
}

function showChatForm(chatType) {
  var chatNameLabel = "";
  if (chatType === "DM") {
    chatNameLabel = "Username/Phone number/Email...";
  } else {
    chatNameLabel = "Enter chat name...";
  }
  var chatArea = document.getElementById("chatName");
  chatArea.setAttribute("placeholder", chatNameLabel);
}

function showCreateChat() {
  var chatArea = document.getElementById("chatArea");
  chatArea.innerHTML = `
        <div class="create-chat-menu">
            <h3>Create Chat</h3>
            <input type="radio" id="dm" name="chatType" value="DM" onclick="showChatForm('DM')">
            <label for="dm">Direct Message</label><br>
            <input checked="checked" type="radio" id="group" name="chatType" value="Group" onclick="showChatForm('Group')">
            <label for="group">Group</label><br>
            <input type="radio" id="channel" name="chatType" value="Channel" onclick="showChatForm('Channel')">
            <label for="channel">Channel</label><br><br>
            <input type="text" id="chatName" class="message-input" placeholder="Enter chat name...">
            <button onclick="createChat()">Create Chat</button>
        </div>`;
}


async function createChat() {
  const type = document.querySelector('input[name="chatType"]:checked').value;
  var chatName = document.getElementById("chatName").value;

  var new_id;
  if (type == "DM") {
    try {
      new_id = await api.contacts.find_user_username(chatName);
    } catch (e) {
      try {
        new_id = await api.contacts.find_user_phonenumber(chatName);
      } catch (e) {
        try {
          new_id = await api.contacts.find_user_email(chatName);
        } catch (e) {
          alert("Cannot find user with: " + chatName);
          return;
        }
      }
    }
    try {
      new_id = await api.chats.create_dm(new_id);
    } catch (e) {
      alert("Cant create DM.. Already made?");
      return;
    }
  } else if (type == "Group") {
    try {
      new_id = await api.chats.create_group(chatName);
    } catch (e) {
      alert("Cant create more groups!");
      return;
    }
  } else {
    try {
      new_id = await api.chats.create_channel(chatName);
    } catch (e) {
      alert("Cant create more channels!");
      return;
    }
  }

  await reload_chat_data();
  document.getElementById("cid" + new_id).onclick();
}

window.userCache = new Map();
async function getUser(user_id) {
  if (window.userCache.get(user_id) == null) {
    window.userCache.set(
      user_id,
      await api.users.get_user(user_id)
    );
  }
  return window.userCache.get(user_id);
}

function sortList(ul) {
  var new_ul = ul.cloneNode(false);

  // Add all lis to an array
  var lis = [];
  for (var i = ul.childNodes.length; i--;) {
    if (ul.childNodes[i].nodeName === 'LI')
      lis.push(ul.childNodes[i]);
  }

  // Sort the lis in descending order
  lis.sort(function (a, b) {
    return parseInt(b.childNodes[0].data, 10) -
      parseInt(a.childNodes[0].data, 10);
  });

  // Add them into the ul in order
  for (var i = 0; i < lis.length; i++)
    new_ul.appendChild(lis[i]);
  ul.parentNode.replaceChild(new_ul, ul);
}


async function showChat(name, chatData) {

  try {
    if (window.EventSource != null)
      window.evtSource.close()
  } catch (e) { }

  var chatArea = document.getElementById("chatArea");
  chatArea.innerHTML = `
        <div class="chat-header">${name}</div>
        <ul style="overflow-x:scroll" class="chat-messages" id="chatMessages">
            <!-- Chat messages for ${name} will be displayed here -->
        </ul>
        <input type="text" id="messageInput" class="message-input" placeholder="Type your message...">
        `;

  document.getElementById("messageInput").addEventListener("keypress", (e) => {
    if (!e.shiftKey && e.keyCode == 13) {
      sendMessage(chatData);
    }
  });
  const evtSource = new EventSource("/database/chat_events/" + chatData.chat_id);
  const eventMap = new Map();
  eventMap.set("NewMessage", async (e) => {
    var message = await api.messages.get_message(e.id);
    addMessageEvent(message, chatData);
  });
  eventMap.set("MessageDeleted", async (e) => {
    deleteMessageEvent(e.id);
  });
  eventMap.set("MessageEdited", async (e) => {
    var message = await api.messages.get_message(e.id);
    updateMessageEvent(message);
  });

  eventMap.set("UserUpdated", async (e) => {
    await refreshUserList(chatData);
  });
  eventMap.set("UserJoined", async (e) => {
    await refreshUserList(chatData);
  });
  eventMap.set("UserLeft", async (e) => {
    await refreshUserList(chatData);
  });

  evtSource.onmessage = async (e) => {
    const event = JSON.parse(e.data);
    (eventMap.get(event.tag))(event);
  };

  (await api.messages.get_messages(chatData.chat_id)).slice().reverse().forEach(message => addMessageEvent(message, chatData))

  window.evtSource = evtSource;
}

async function refreshUserList(chatData) {

}

function deleteMessageEvent(mid) {
  document.getElementById("mid" + mid).outerHTML = "";
}

async function updateMessageEvent(message) {
  const element = document.getElementById("mid" + message.message_id);
  if (element != null) {
    element.innerHTML = await generateInner(message);
  }
}

function dateFormat(date) {
  const formatter = new Intl.DateTimeFormat('en-US', { day: '2-digit', month: '2-digit', year: 'numeric' });
  return formatter.format(date);
}

async function deleteMessage(message_id) {
  try {
    await api.messages.delete_message(message_id);
  } catch (e) {
    alert("you aren't able to delete that message");
  }
}

async function editMessage(message_id) {

}

async function replyMessage(message_id) {

}

async function generateInner(message) {
  const user = await getUser(message.sender_id);

  let date;
  if (message.last_edited == null) {
    date = dateFormat(new Date(message.posted));
    date = `<span class="date">${date}</span>`;
  } else {
    date = dateFormat(new Date(message.last_edited));
    date = `<span class="date">edited: ${date}</span>`;
  }

  let pfp;
  if (user.pfp_file_id == null) {
    pfp = "https://upload.wikimedia.org/wikipedia/commons/thumb/2/2c/Default_pfp.svg/2048px-Default_pfp.svg.png"
  } else {
    pfp = "/database/attachments/" + user.pfp_file_id;
  }

  let views = "";
  if (message.views != null) {
    views = `<span class="views">views: ${message.views}</span>`
  }


  return `
  <div class="message-container">
    <!-- Profile picture -->
    <img src="${pfp}" alt="Profile Picture" class="pfp">

    <div class="message-details">
        <div class="msg-info">
          <!-- Username -->
          <span class="username">${user.display_name}</span>
          <!-- Date uploaded -->
          ${date}
          ${views}

          <!-- Options -->
          <div class="options">
              <button onclick='deleteMessage(${message.message_id})'>Delete</button>
              <button>Edit</button>
              <button>Reply</button>
          </div>
        </div>
        <!-- Message contents -->
        <div class="message-content">
            <p>${message.message}</p>
        </div>

    </div>
  </div>`;
}

async function addMessageEvent(message, chatData) {

  const chatArea = document.getElementById("chatArea");
  var chatMessages = document.getElementById("chatMessages");
  var newMessage = document.createElement("div");
  newMessage.id = "mid" + message.message_id;

  newMessage.innerHTML = await generateInner(message);
  chatMessages.appendChild(newMessage);
  document.getElementById("chatMessages").scroll(0, 999999999);
  if (chatData.tracks_views) {
    await api.messages.view_message(message.message_id);
  }
}

async function sendMessage(chatData) {
  var messageInput = document.getElementById("messageInput");
  var message = messageInput.value;
  let num = await api.messages.send_message(message, chatData.chat_id);

  messageInput.value = "";
}



async function reload_chat_data() {
  window.chatData = await api.chats.list_chats();

  const dmArea = document.getElementById("dm-chats");
  const groupArea = document.getElementById("group-chats");
  const channelArea = document.getElementById("channel-chats");

  dmArea.innerHTML = "";
  groupArea.innerHTML = "";
  channelArea.innerHTML = "";


  for (const data of chatData) {
    if (data.seconary != null) {
      let other_id = data.owner == userData.user_id ? data.seconary : data.owner;
      let name = await api.users.get_username(other_id);
      dmArea.innerHTML += `<li class="chat-list-item" id='cid${data.chat_id}' onclick='showChat(${JSON.stringify(name)}, ${JSON.stringify(data)})'>${name}</li>`;
    } else if (data.send_priv == 0) {
      let name = data.name;
      groupArea.innerHTML += `<li class="chat-list-item" id='cid${data.chat_id}' onclick='showChat(${JSON.stringify(name)}, ${JSON.stringify(data)})'>${name}</li>`;
    } else {
      let name = data.name;
      channelArea.innerHTML += `<li class="chat-list-item" id='cid${data.chat_id}' onclick='showChat(${JSON.stringify(name)}, ${JSON.stringify(data)})'>${name}</li>`;
    }
  }
}

async function reload_user_data() {
  window.userData = await api.users.who_am_i();
  if (window.userData == null) {
    window.location.href = 'login.html'; // Redirect to login page
  }
}

async function user_event_listener() {

}




document.addEventListener("DOMContentLoaded", async function (event) {
  await reload_user_data();
  await reload_chat_data();
  await user_event_listener();
  showSettings();
});