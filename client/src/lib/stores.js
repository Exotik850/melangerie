import { browser } from "$app/environment";
import { checkUser } from "$lib";
import { writable, type Writable } from "svelte/store";

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
export const incomingMessages: Writable<State> = writable<State>({
  requests: [],
});
export const connect = (url: URL) => {
  $incomingMessages.socket = new WebSocket(url);
  $incomingMessages.socket.addEventListener("message", async (message: any) => {
    message = await message.data.text();
    const data: Message = JSON.parse(message);
    console.log(data);
    incomingMessages.update((state) => ({
      ...state,
      requests: [data].concat(state.requests),
    }));
  });
};

export const sendMessage = (message: Message) => {
  if ($incomingMessages.socket) {
    $incomingMessages.socket.send(JSON.stringify(message));
    console.log(message, "Sent");
  } else {
    console.log("No socket connection");
  }
};
