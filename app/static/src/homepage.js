

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
  const user = await page.session.getUser(message.sender_id);

  let date;
  if (message.last_edited == null) {
    date = api.formatDate(new Date(message.posted));
    date = `<span class="date">${date}</span>`;
  } else {
    date = api.formatDate(new Date(message.last_edited));
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
              <button onclick='editMessage(${message.message_id})'>Edit</button>
              <button onclick='replyMessage(${message.message_id})'>Reply</button>
          </div>
        </div>
        <!-- Message contents -->
        <div class="message-content">
            <p>${message.message}</p>
        </div>

    </div>
  </div>`;
}






class Chat{

  current_chat_id;

  constructor(){
    this.chatArea = document.createElement("div");
    this.chatArea.classList.add("chat");
    this.chatArea.id = "chatArea";
    this.chatArea.innerHTML = `
          <div id="chatName" class="chat-header"></div>
          <ul style="overflow-x:scroll" class="chat-messages" id="chatMessages">
          </ul>
          <input type="text" id="messageInput" class="message-input" placeholder="Type your message...">
          `;

    const input = this.chatArea.querySelector("#messageInput");
    input.addEventListener("keypress", async (e) => {
      if (!e.shiftKey && e.keyCode == 13) {
        var message = input.value;
        let num = await api.messages.send_message(message, this.current_chat_id);
        input.value = "";
      }
    });
  }

  async show(chat_id){
    await page.session.subscribe_chat(chat_id);
    this.current_chat_id = chat_id;
    document.getElementById("content").innerHTML = "";
    document.getElementById("content").appendChild(page.currentChat.chatArea);
    await this.realod_info(chat_id);

    var chatMessages = this.chatArea.querySelector("#chatMessages");
    chatMessages.innerHTML = "";

    for (const message of await api.messages.get_messages(this.current_chat_id)){
      this.addOldMessage(message)
    }
  }

  deleteMessage(mid) {
    document.getElementById("mid" + mid).outerHTML = "";
  }
  
  async updateMessage(message) {
    const element = document.getElementById("mid" + message.message_id);
    if (element != null) {
      element.innerHTML = await generateInner(message);
    }
  }

  async addOldMessage(message){
    // const chatArea = document.getElementById("chatArea");
    var chatMessages = this.chatArea.querySelector("#chatMessages");
    var newMessage = document.createElement("div");
    chatMessages.prepend(newMessage);
    newMessage.id = "mid" + message.message_id;

    newMessage.innerHTML = await generateInner(message);
    chatMessages.scroll(0, 999999999);

    if (page.session.getChat(this.current_chat_id).tracks_views) {
      await api.messages.view_message(message.message_id);
    }
  }

  async addMessage(message) {

    var chatMessages = this.chatArea.querySelector("#chatMessages");
    var newMessage = document.createElement("div");
    chatMessages.appendChild(newMessage);
    newMessage.id = "mid" + message.message_id;
  
    newMessage.innerHTML = await generateInner(message);
    chatMessages.scroll(0, 999999999);
  
    if (page.session.getChat(this.current_chat_id).tracks_views) {
      await api.messages.view_message(message.message_id);
    }
  }

  async realod_info(){
    const chat = await page.session.getChat(this.current_chat_id);
    
    this.chatArea.querySelector("#chatName").innerHTML = chat.display_name;
  }

  async event(e){
    switch (e.tag){
      case "UserJoined":
        break;
      case "UserLeft":
          break;
      
      case "ChatUpdated":
        await this.realod_info();
        break;     
      case "NewMessage":
          await this.addMessage(await page.session.getMessage(e.message))
          break;
      case "MessageEdited":
          await this.editMessage(await page.session.getMessage(e.message))  
          break;
      case "MessageDeleted":
          await this.deleteMessage(e.message)
          break;
    }
  }
}

class UserSettings{

  constructor(){
    this.settingsArea = document.createElement("div");
    this.settingsArea.classList.add("chat");
    this.settingsArea.id = "userSettings";
    
    this.settingsArea.innerHTML = `
          <div class="settings-menu">
  
          <form id="profileForm">
              <img id="pfp" alt="Profile Picture" class="pfp">
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
              
              <button type="button" onclick="page.userSettings.update()">Update</button>
            </form>
            <button onclick="api.users.logout()">Logout</button>
          </div>`;
  
    const dropzone = this.settingsArea.querySelector('#fileDropzone');
    const fileInput = this.settingsArea.querySelector('#fileInput');
  
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
    }
  }

  async reload(){
    for (const key of Object.keys(page.session.user)){
        const el = this.settingsArea.querySelector("#"+key)
        if(el != null){
            el.value = page.session.user[key]
        }
    }

    let pfp;
    if (page.session.user.pfp_file_id == null) {
      pfp = "https://upload.wikimedia.org/wikipedia/commons/thumb/2/2c/Default_pfp.svg/2048px-Default_pfp.svg.png"
    } else {
      pfp = "/database/attachments/" + page.session.user.pfp_file_id;
    }
    this.settingsArea.querySelector("#pfp").src = pfp;
  }
  
  async update() {
    // Get form data
    const form = this.settingsArea.querySelector('#profileForm');
    const formData = new FormData(form);
    
    var object = {};
      formData.forEach((value, key) => object[key] = value);  
  
    await api.users.update_user(object);
  }
}


class Page{
  currentChat = new Chat();
  userSettings = new UserSettings();

  constructor() {
    
  }

  async showChat(chat_id){
    this.currentChat.show(chat_id);
  }

  async showCreateChat(){
    page.session.unsubscribe_chat();  
    document.getElementById("content").innerHTML = `
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

  async showUserSettings(){
    page.session.unsubscribe_chat();

    await this.userSettings.reload();
    document.getElementById("content").innerHTML = "";
    document.getElementById("content").appendChild(this.userSettings.settingsArea);
  }

  async reloadChatList() {
    const chats = await page.session.getChats();
  
    const dmArea = document.getElementById("dm-chats");
    const groupArea = document.getElementById("group-chats");
    const channelArea = document.getElementById("channel-chats");
  
    dmArea.innerHTML = "";
    groupArea.innerHTML = "";
    channelArea.innerHTML = "";
  
    for (const chat of chats) {
      const content = `<li class="chat-list-item" onclick='page.showChat(${chat.chat_id})'>${chat.display_name}</li>`;
      if (chat.kind == "dm"){
        dmArea.innerHTML += content;
      }else if (chat.kind=="group"){
        groupArea.innerHTML += content;
      }else{
        channelArea.innerHTML += content;
      }
    }
  }

  async begin(){
    this.session = new api.session.Session();
    
    this.session.addSelfListener(async (e) => {
      switch (e.tag){
        case "WhoAmI":
          await this.userSettings.reload();
          break;
        case "UserDeleted":
            window.location.href = 'login.html';
            break;
        case "UserUpdated":
            await this.userSettings.reload();
            break;

        case "UserJoined":
            await this.reloadChatList();
            break;
        case "UserLeft":
            await this.reloadChatList();
            break;
      }
    });

    this.session.addOtherListener((e) => {

    });

    this.session.addChatListener((e) => this.currentChat.event(e));
    
    await this.session.begin();
    await this.reloadChatList();
    await this.showUserSettings();
  }
}

const page = new Page();
document.addEventListener("DOMContentLoaded", async function (event) {
  page.begin();
});


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