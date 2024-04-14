<script lang="ts">
  import { get } from "svelte/store";
  import { onMount } from "svelte";
  import {
    connect,
    incomingMessages,
    listRooms,
    messageStore,
    sendMessage,
    token_store,
  } from "$lib/stores";
  import { host, type JWT } from "$lib";
  import Modal from "../components/Modal.svelte";
  let token = get(token_store);
  let uname: null | JWT;
  let message = "";
  let rooms: null | string[];
  let selectedRoom = "";
  onMount(async () => {
    window.onunload = () => {
      if ($incomingMessages.socket) {
        $incomingMessages.socket.close();
      }
    };
    if (token) {
      connect(new URL(host + "/chat/connect"));
      uname = JSON.parse(atob(token.split(".")[1]));
      console.log("Found ", uname);
      rooms = await listRooms();
    }
  });
  const createRoom = async () => {
    const roomName = prompt("Enter the room name:");
    if (roomName) {
      await fetch(host + `/chat/create/${roomName}/` + uname?.name, { method: "POST", headers: { "authorization": token } });
      rooms = await listRooms();
    }
  };
  const joinRoom = (room: string) => {
    selectedRoom = room;
  };
</script>

<main>
  {#if token}
    {#if uname}
      <div class="chat-container">
        <div class="sidebar">
          <h2>Welcome {uname.name}</h2>
          <button on:click={createRoom}>Create Room</button>
          <h3>Rooms</h3>
          <ul>
            {#if rooms != null}
              {#each rooms as room}
                <li><button on:click={() => joinRoom(room)}>{room}</button></li>
              {/each}
            {/if}
          </ul>
        </div>
        <div class="chat-window">
          {#if selectedRoom}
            <h2>{selectedRoom}</h2>
            <ul class="message-list">
              {#if $messageStore[selectedRoom]}
                {#each $messageStore[selectedRoom] as message}
                  <li><strong>{message.sender}:</strong> {message.content}</li>
                {/each}
              {/if}
            </ul>
            <div class="message-input">
              <input
                type="text"
                bind:value={message}
                placeholder="Type a message..."
              />
              <button
                on:click={() =>
                  sendMessage({
                    action: "Message",
                    data: {
                      sender: uname?.name,
                      room: selectedRoom,
                      content: message,
                      timestamp: Date.now(),
                    },
                  })}
              >
                Send
              </button>
            </div>
          {:else}
            <p>Select a room to start chatting.</p>
          {/if}
        </div>
      </div>
    {:else}
      <h1>Loading...</h1>
    {/if}
  {:else}
    <Modal/>
  {/if}
</main>

<style lang="scss">
  .chat-container {
    display: flex;
    height: 100vh;
  }
  .sidebar {
    width: 200px;
    background-color: #f0f0f0;
    padding: 20px;
    h2 {
      margin-bottom: 20px;
    }
    button {
      margin-bottom: 10px;
    }
    ul {
      list-style: none;
      padding: 0;
      li {
        margin-bottom: 5px;
        button {
          width: 100%;
          text-align: left;
        }
      }
    }
  }
  .chat-window {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 20px;
    h2 {
      margin-bottom: 10px;
    }
    .message-list {
      flex: 1;
      overflow-y: auto;
      list-style: none;
      padding: 0;
      li {
        margin-bottom: 10px;
      }
    }
    .message-input {
      display: flex;
      margin-top: 20px;
      input {
        flex: 1;
        padding: 10px;
        border-radius: 5px;
        border: 1px solid #ccc;
      }
      button {
        margin-left: 10px;
      }
    }
  }
</style>
