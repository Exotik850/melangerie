<script lang="ts">
  import { get } from "svelte/store";
  import Modal from "../components/Modal.svelte";
  import { onMount } from "svelte";
  import {
    connect,
    incomingMessages,
    messageStore,
    sendMessage,
    token_store,
  } from "$lib/stores";
  import { host } from "$lib";

  let token = get(token_store);
  let uname: null | string;
  let message = "";

  onMount(async () => {

    window.onunload = () => {
      if ($incomingMessages.socket) {
        $incomingMessages.socket.close();
      }
    };

    if (token) {
      await fetch(host + "/create/apple", { method: "POST" });
      connect(new URL(host + "/connect"));
      uname = JSON.parse(atob(token.split(".")[1]))
      console.log("Found ", uname)
    }
  });
</script>

<main>
  {#if token}
    You're logged in

    <input type="text" bind:value={message} />
    <button
      on:click={() =>
        sendMessage({
          action: "Message",
          data: {
            sender: token,
            room: "general",
            content: message,
            timestamp: Date.now(),
          }
        })}
    >
      Send
    </button>

    <ul>
      {#each Object.entries($messageStore) as [room, messages]}
        <h2>{room}</h2>

        {#each messages as message}
          <li>{message.sender} : {message.content}</li>
        {/each}
        <br />
      {/each}
    </ul>
  {:else}
    <Modal />
  {/if}
</main>
