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

async function upload_image(name, src){
    const arrayBuffer = await (await fetch(src)).arrayBuffer();
    // console.log(arrayBuffer);
    return await files.upload_file(name, arrayBuffer);
}

export async function test(){
    const ivy_image = await upload_image("ivy.png", "https://media.discordapp.net/attachments/900461634030567457/1233969800384876607/S5e21_Braco_and_PB.png?ex=662f076e&is=662db5ee&hm=36a8e8797e3bc86120e602e3c6442e715efea53f3565e7069bc6cd83b69ab8b7&=&format=webp&quality=lossless&width=80&height=80");
    const brett_image = await upload_image("brett.jpg", "https://media.discordapp.net/attachments/1163024155587911782/1234027046708052030/fleamarketsocialist.jpg?ex=662f3cbf&is=662deb3f&hm=d40dc969ecac224fcc2f7c96a7d9d76717e545764ba8b04b893d76505349e652&=&format=webp&width=157&height=157");
    const parkers_image = await upload_image("parker.jpg", "https://media.discordapp.net/attachments/1163024155587911782/1233279992146825237/20240424_004316.jpg?ex=662f27ff&is=662dd67f&hm=6deeb8c0911ea8b7314767b18a78cc1a07b23ac1db36d3273b49827fd66de087&=&format=webp&width=285&height=617");
    


    const ivy = await users.create_user({name: "ivy", email: "ivy", location: "ivy", password: "ivy", phone_number: "123", username: "ivy", location: "ivy"});
    const parker = await users.create_user({name: "parker", email: "parker", location: "parker", password: "parker", phone_number: "1233", username: "parker", location: "parker"});
    const brett = await users.create_user({name: "brett", email: "brett", location: "brett", password: "brett", phone_number: "brett", username: "brett", location: "brett"});
    const little = await users.create_user({name: "little", email: "little", location: "little", password: "little", phone_number: "little", username: "little", location: "little"});
    
    await users.login("ivy", "ivy");
    await users.update_user_pfp(ivy_image);
    await users.login("parker", "parker");
    await users.update_user_pfp(parkers_image);
    await users.login("brett", "brett");
    await users.update_user_pfp(brett_image);
    
    await users.login("parker", "parker");
    let ivy_parker_dm = await chats.create_dm(ivy);
    await messages.send_message("hello!", ivy_parker_dm);


    let parker_brett_dm = await chats.create_dm(brett);
    await messages.send_message("you're strange!", parker_brett_dm);

    await users.login("ivy", "ivy");
    const mid1 = await messages.send_message("hiiii!", ivy_parker_dm);
    const mid2 = await messages.send_message("byr", ivy_parker_dm);
    await messages.update_message("bye", mid2);
    await messages.delete_message(mid1);

    await users.login("parker", "parker");
    const group_id = await chats.create_group("BRUH!");
    await messages.send_message("asdlfjhasdlkfgjhsad", group_id);

    await users.login("ivy", "ivy");
    await chats.join_chat(group_id);
    await messages.send_message("hiiii!", group_id);


    await users.login("brett", "brett");
    await chats.join_chat(group_id);
    await messages.send_message("I hate you all", group_id);

    let cid = await chats.create_channel("asldkajsdl");
    await messages.send_message("asdjhkasdjh", cid);

}