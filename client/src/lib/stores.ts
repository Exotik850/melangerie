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
type State = {
  socket?: WebSocket,
  requests: Array<Message>,
};
type Message = {
  sender: string,
  room: string,
  content: string,
  timestamp: number,
};
// Create a new store with the given data.
export const incomingMessages = writable<State>({
  requests: [],
});
export const connect = (url: URL) => {
  let ws = new WebSocket(url);
  ws.addEventListener("message", async (message: any) => {
    message = await message.data.text();
    const data: Message = JSON.parse(message);
    console.log(data);
    incomingMessages.update((state) => ({
      ...state,
      requests: [data].concat(state.requests),
    }));
  });
  incomingMessages.update((state) => ({
    ...state,
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
