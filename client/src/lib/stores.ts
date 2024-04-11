import { browser } from "$app/environment";
import { checkUser } from "$lib";
import { writable, get } from "svelte/store";

function get_uname() {
  let uname = sessionStorage.getItem("OCusername");
  if (uname && checkUser(uname)) {
    return uname;
  } else {
    return "";
  }
}
export const uname_store = writable((browser && get_uname()) || "");
export const messageStore = writable({} as Record<string, Message[]>);


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
  let ws = new WebSocket(url);
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

export const sendMessage = (message: Message) => {
  let socket = get(incomingMessages).socket;
  if (socket) {
    socket.send(JSON.stringify(message));
    console.log(message, "Sent");
  } else {
    console.log("No socket connection");
  }
};
