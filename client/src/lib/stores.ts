import { browser } from "$app/environment";
import { checkUser } from "$lib";
import { writable, get } from "svelte/store";

let access_token: null | string = null;
export const token_store = writable((browser && get_token()) || "");
token_store.subscribe((val) => {
  console.log("Token store updated", val);
})
export const messageStore = writable({} as Record<string, Message[]>);

function get_token() {
  let token = localStorage.getItem("OCTOKEN") || null;
  return token;
}

type State = {
  socket?: WebSocket,
};
type Payload = {
  action: string,
  data: any,
};
type Message = {
  sender: string,
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
    let token = get(token_store).toString();
    console.log("Sending token", token);
    ws.send(token);
  };
  ws.onclose = () => {
    console.log("Connection closed");

  }
  // ws.send(token);
  ws.addEventListener("message", async (message: any) => {
    message = await message.data.text();
    const data: Message = JSON.parse(message);
    console.log(data);
    messageStore.update((state) => {
      state[data.room] = state[data.room] || [];
      state[data.room].push(data);
      return state;
    });
  });
  incomingMessages.update((state) => ({
    socket: ws,
  }));
};

export const sendMessage = (message: Payload) => {
  let socket = get(incomingMessages).socket;
  if (socket) {
    socket.send(JSON.stringify(message));
    console.log(message, "Sent");
  } else {
    console.log("No socket connection");
  }
};
