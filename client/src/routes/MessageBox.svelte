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
      <!-- Add user to this room -->
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
</style>
