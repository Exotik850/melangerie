import { browser } from "$app/environment";
import { checkUser, host } from "$lib";
import { writable, get, derived } from "svelte/store";

let access_token: null | string = null;
export const token_store = writable("");
export let uname = derived(token_store, (token) => {
  if (!token) return "";
  return JSON.parse(atob(token.split(".")[1])).name;
});
token_store.subscribe((val) => {
  console.log("Token store updated", val);
})
export const messageStore = writable<Record<string, Message[]>>({});

// function get_token() {
//   let token = localStorage.getItem("OCTOKEN") || null;
//   return token;
// }

type State = {
  socket?: WebSocket,
};
type Payload = {
  action: string,
  data: any,
};
type Message = {
  sender?: string,
  room: string,
  content: string,
  timestamp: number,
};
// Create a new store with the given data.
export const incomingMessages = writable<State>({});
export const connect = (url: URL) => {
  let token = get(token_store);
  if (!token) {
    console.log("No token found");
    return;
  }
  let ws = new WebSocket(url);
  ws.onopen = () => {
    console.log("Connected to server");
    let token = get(token_store);
    console.log("Sending token", token);
    ws.send(token);
  };
  ws.onclose = () => {
    console.log("Connection closed");
  }
  // ws.send(token);
  ws.addEventListener("message", async (message: any) => {
    message = await message.data.text();
    const data: Payload = JSON.parse(message);
    console.log(data);
    handlePayload(data);
  });
  incomingMessages.update((state) => ({
    socket: ws,
  }));
};

function handlePayload(payload: Payload) {
  switch (payload.action) {
    case "Message":
      messageStore.update((state) => {
        state[payload.data.room] = state[payload.data.room] || [];
        state[payload.data.room].push(payload.data);
        return state;
      });
      break;
    case "Join":
      const room = payload.data[0];
      messageStore.update((state) => {
        state[room] = state[room] || [];
        // if (payload.data[1] === get(uname)) {
        //   return state;
        // };
        state[room].push({
          sender: "Server",
          content: `${payload.data[1]} joined the room`,
          timestamp: Date.now(),
        });
        return state;
      })

    default:
      break;
  }
}

export const sendMessage = (message: Payload) => {
  let socket = get(incomingMessages).socket;
  if (socket) {
    socket.send(JSON.stringify(message));
  } else {
    console.log("No socket connection");
  }
};

export const listRooms = async () => {
  let token = get(token_store);
  if (!token) {
    console.log("No token found");
    return;
  }
  let res = await fetch(host + "/chat/list", {
    headers: {
      authorization: token,
    },
  });
  let data = await res.json();
  return data;
}