<!-- <script lang="ts">
  import SelectUser from "./SelectUser.svelte";
  import {
    incomingMessages,
    messageStore,
    selectedRoom,
    sendMessage,
    usersStore,
    token_store,
    uname,
  } from "$lib/stores";
  import { host } from "$lib";
  import { toast } from "svelte-french-toast";
  import { tick } from "svelte";
  let message = "";
  function addUser() {
    // Open a modal to select from available users
    sendMessage({ action: "ListUsers" });
    // delay for 100 ms
    setTimeout(() => {
      console.log("Users: ", $usersStore);
      $usersStore = $usersStore.filter((user) => user !== $uname);
      if ($usersStore.length === 0) {
        toast.error("No users found!");
        return;
      }
      selectingUser = true;
    }, 100);
  }
  let textInput: HTMLInputElement;
  async function sendMessageFromUser() {
    sendMessage({
      action: "Message",
      data: {
        sender: $uname,
        room: $selectedRoom,
        content: message,
        timestamp: Date.now(),
      },
    });
    message = "";
    await tick();
    textInput.focus();
  }
  let selectingUser = false;
  function leaveRoom() {
    if (!$selectedRoom) {
      toast.error("No room selected!");
      return;
    }
    sendMessage({
      action: "Leave",
      data: $selectedRoom,
    });
    messageStore.update((store) => {
      delete store[$selectedRoom];
      return store;
    });
    selectedRoom.set(null);
  }
</script>

<SelectUser
  callback={(user) => {
    selectingUser = false;
    if (user && $selectedRoom && $incomingMessages.socket) {
      $incomingMessages.socket.send(
        JSON.stringify({
          action: "Add",
          data: [$selectedRoom, user],
        })
      );
    }
  }}
  bind:open={selectingUser}
/>

<div class="chat-window">
  {#if $selectedRoom}
    <div class="chat-header">
      <h2>{$selectedRoom}</h2>
      <div class="buttons">
        <button on:click={addUser}>Add User</button>
        <button class="leave" on:click={leaveRoom}>Leave</button>
      </div>
    </div>
    <ul class="message-list">
      {#if $messageStore[$selectedRoom]}
        {#each $messageStore[$selectedRoom] as message}
          <li><strong>{message.sender}:</strong> {message.content}</li>
        {/each}
      {/if}
    </ul>
    <form class="message-input">
      <input
        type="text"
        bind:this={textInput}
        bind:value={message}
        placeholder="Type a message..."
      />
      <button on:click={sendMessageFromUser} type="submit"> Send </button>
    </form>
  {:else}
    <p>Select a room to start chatting.</p>
  {/if}
</div>

<style lang="scss">
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
  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: #f2f2f2;
    padding: 10px;
    border-bottom: 1px solid #ccc;
    h2 {
      margin: 0;
      font-size: 18px;
      font-weight: bold;
    }
    button {
      padding: 5px 10px;
      border-radius: 5px;
      background-color: #007bff;
      color: #fff;
      border: none;
      cursor: pointer;
      transition: background-color 0.3s ease;
      &:hover {
        background-color: #0056b3;
      }
    }
  }

  .buttons {
    display: flex;
    button {
      margin-left: 10px;
    }
  }
</style> -->
<script lang="ts">
  import SelectUser from "./SelectUser.svelte";
  import {
    incomingMessages,
    messageStore,
    selectedRoom,
    sendMessage,
    usersStore,
    token_store,
    uname,
  } from "$lib/stores";
  import { host } from "$lib";
  import { toast } from "svelte-french-toast";
  import { tick } from "svelte";

  let message = "";

  function addUser() {
    sendMessage({ action: "ListUsers" });
    setTimeout(() => {
      console.log("Users: ", $usersStore);
      $usersStore = $usersStore.filter((user) => user !== $uname);
      if ($usersStore.length === 0) {
        toast.error("No users found!");
        return;
      }
      selectingUser = true;
    }, 100);
  }

  let textInput: HTMLInputElement;

  function sendMessageFromUser() {
    sendMessage({
      action: "Message",
      data: {
        sender: $uname,
        room: $selectedRoom,
        content: message,
        timestamp: Date.now() / 1000,
      },
    });
    message = "";
    setTimeout(() => {
      textInput.focus();
    }, 100);
  }

  let selectingUser = false;

  function leaveRoom() {
    if (!$selectedRoom) {
      toast.error("No room selected!");
      return;
    }
    sendMessage({
      action: "Egress",
      data: {room_id: $selectedRoom, user_id: $uname, action: "Leave"},
    });
    messageStore.update((store) => {
      delete store[$selectedRoom];
      return store;
    });
    selectedRoom.set(null);
  }
</script>

<SelectUser
  callback={(user) => {
    selectingUser = false;
    if (user && $selectedRoom && $incomingMessages.socket) {
      $incomingMessages.socket.send(
        JSON.stringify({
          action: "Egress",
          data: {
            room_id: $selectedRoom,
            user_id: user,
            action: "Join",
          },
        })
      );
    }
  }}
  bind:open={selectingUser}
/>
<div class="chat-window">
  {#if $selectedRoom}
    <div class="chat-header">
      <h2>{$selectedRoom}</h2>
      <div class="buttons">
        <button class="add-user" on:click={addUser}>
          <i class="fas fa-user-plus"></i>
        </button>
        <button class="leave" on:click={leaveRoom}>
          <i class="fas fa-sign-out-alt"></i>
        </button>
      </div>
    </div>
    <div class="message-container">
      <ul class="message-list">
        {#if $messageStore[$selectedRoom]}
          {#each $messageStore[$selectedRoom] as message}
            <li class="message">
              <span class="sender">{message.sender}:</span>
              <span class="content">{message.content}</span>
            </li>
          {/each}
        {/if}
      </ul>
      <form class="message-input">
        <input
          type="text"
          bind:this={textInput}
          bind:value={message}
          placeholder="Type a message..."
        />
        <button
          class="send-button"
          on:click={sendMessageFromUser}
          type="submit"
        >
          <i class="fas fa-paper-plane"></i>
        </button>
      </form>
    </div>
  {:else}
    <div class="no-room-selected">
      <p>Select a room to start chatting.</p>
    </div>
  {/if}
</div>

<style lang="scss">
  .chat-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 75%;
    background-color: #f5f5f5;
    border-radius: 10px;
    overflow: hidden;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);

    .chat-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 20px;
      background-color: #ffffff;
      border-bottom: 1px solid #e0e0e0;

      h2 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
        color: #333333;
      }

      .buttons {
        display: flex;
        align-items: center;

        button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 40px;
          height: 40px;
          margin-left: 10px;
          background-color: transparent;
          border: none;
          border-radius: 50%;
          color: #777777;
          font-size: 16px;
          cursor: pointer;
          transition: background-color 0.3s ease;

          &:hover {
            background-color: #f0f0f0;
          }

          &.add-user {
            color: #4caf50;
          }

          &.leave {
            color: #f44336;
          }
        }
      }
    }

    .message-container {
      flex: 1;
      display: flex;
      flex-direction: column;
      padding: 20px;
      overflow-y: auto;

      .message-list {
        flex: 1;
        overflow-y: auto;
        list-style: none;
        padding: 0;
        margin: 0;

        .message {
          display: flex;
          align-items: flex-start;
          margin-bottom: 10px;

          .sender {
            font-weight: 600;
            margin-right: 5px;
            color: #333333;
          }

          .content {
            color: #555555;
          }
        }
      }

      .message-input {
        display: flex;
        align-items: center;
        margin-top: 20px;

        input {
          flex: 1;
          padding: 10px;
          border-radius: 20px;
          border: 1px solid #e0e0e0;
          font-size: 14px;
          outline: none;
        }

        .send-button {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 40px;
          height: 40px;
          margin-left: 10px;
          background-color: #4caf50;
          border: none;
          border-radius: 50%;
          color: #ffffff;
          font-size: 16px;
          cursor: pointer;
          transition: background-color 0.3s ease;

          &:hover {
            background-color: #45a049;
          }
        }
      }
    }

    .no-room-selected {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 18px;
      color: #777777;
    }
  }
</style>
