import { browser } from "$app/environment";
import { checkUser, host } from "$lib";
import { writable, get, derived } from "svelte/store";
import { toast } from "svelte-french-toast";

let access_token: null | string = null;
export const token_store = writable("");
export let uname = derived(token_store, (token) => {
  if (!token) return "";
  return JSON.parse(atob(token.split(".")[1])).name;
});
export const messageStore = writable<Record<string, Message[]>>({});
export const selectedRoom = writable<string | null>(null);
export const usersStore = writable<string[]>([]);
export const tabHidden = writable(false);
export const timedIn = writable(false);
// let sound = new Audio("/notification.mp3");
let sound: HTMLAudioElement | null = null;

// function get_token() {
//   let token = localStorage.getItem("OCTOKEN") || null;
//   return token;
// }

type State = {
  socket?: WebSocket;
};
type Payload = {
  action: string;
  data?: any;
};
type Message = {
  sender?: string;
  room: string;
  content: string;
  timestamp: number;
};
type Added = {
  room: string;
  adder?: string;
  added: string;
  timestamp: number;
}
type TimeInOut = {
  note?: string;
}

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
    ws.send(JSON.stringify({ action: "CheckTime" }));
  };
  ws.onclose = () => {
    console.log("Connection closed");
    toast.error("Connection closed");
  };
  // ws.send(token);
  ws.addEventListener("message", async (message: any) => {
    console.log("Received:", message.data);
    if (get(tabHidden)) {
      // play sound
      console.log("Playing sound")
      if (!sound) {
        sound = new Audio("/notification.mp3");
      }
      sound.play();
    }
    const data: Payload = JSON.parse(await message.data.text());
    handlePayload(data);
  });
  incomingMessages.update(() => ({
    socket: ws,
  }));
};

function handlePayload(payload: Payload) {
  switch (payload.action) {
    case "Message":
      const room = payload.data?.room;
      messageStore.update((state) => {
        state[room] = state[room] || [];
        state[room].push(payload.data);
        if (get(selectedRoom) != room && payload.data?.timestamp > Date.now() - 1000 * 60 * 1 ){
          toast.success(`${payload.data?.sender} sent a message in ${room}`);
        }
        return state;
      });
      break;
    case "Added":
      if (!payload.data) return;
      const { room: roomName, added, adder, timestamp } = payload.data;
      messageStore.update((state) => {
        state[roomName] = state[roomName] || [];
        let content;
        if (adder) {
          content = `${adder} added ${added} to the room`;
        } else {
          content = `${added} joined the room`;
        }
        state[roomName].push({
          sender: "Server",
          content: content,
          timestamp: timestamp,
          room: roomName,
        });
        if (get(selectedRoom) != roomName) {
          toast.success(`${added} joined ${roomName}`);
        }
        return state;
      });
    case "List":
      if (!payload.data) return;
      usersStore.set(payload.data);
      break;
    case "TimedIn":
      timedIn.set(payload.data || false);
      break;
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
