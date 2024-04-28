import * as messages from "./messages.js";
import * as chats from "./chats.js";
import * as users from "./users.js";
import * as contacts from "./contacts.js";
import * as admin from "./admin.js";
import * as files from "./files.js";

export {messages, chats, users, contacts, admin, files};

export async function listen(channel_id){
    const evtSource = new EventSource("/database/listen_for_messages/"+channel_id);

    evtSource.onmessage = (event) => {
        console.log(event);
      };
}

export async function test(){
    const arrayBuffer = await (await fetch("https://media.discordapp.net/attachments/1163024155587911782/1233958514343411772/image.png?ex=662efceb&is=662dab6b&hm=f8dcc9a584db06b2f42e37332fb9ded08318006b9484e5a8401fc945a8bf1130&=&format=webp&quality=lossless&width=1100&height=617")).arrayBuffer();
    console.log(arrayBuffer);
    let file_id = await files.upload_file("funny.png", arrayBuffer);
    console.log("/database/attachments/" + file_id + "/funny.png");


    const uid1 = await users.create_user({name: "ivy", email: "ivy", location: "ivy", password: "ivy", phone_number: "123", username: "ivy", location: "ivy"});
    const uid2 = await users.create_user({name: "parker", email: "parker", location: "parker", password: "parker", phone_number: "1233", username: "parker", location: "parker"});
    await users.login("parker", "parker");
    let dm_id = await chats.create_dm(uid1);
    await messages.send_message("hello!", dm_id);

    await users.login("ivy", "ivy");
    const mid1 = await messages.send_message("hiiii!", dm_id);
    const mid2 = await messages.send_message("byr", dm_id);
    await messages.update_message("bye", mid2);
    await messages.delete_message(mid1);

    await users.login("parker", "parker");
    const group_id = await chats.create_group("BRUH!");
    await messages.send_message("asdlfjhasdlkfgjhsad", group_id);

    await users.login("ivy", "ivy");
    await chats.join_chat(group_id);
    await messages.send_message("hiiii!", group_id);

    let cid = await chats.create_channel("asldkajsdl");
    await messages.send_message("asdjhkasdjh", cid);

}