<script lang="ts">
  import { get } from "svelte/store";
  import Modal from "../components/Modal.svelte";
  import { onMount } from "svelte";
  import {
    connect,
    messageStore,
    sendMessage,
    uname_store,
  } from "$lib/stores";
  import { host } from "$lib";

  let uname = get(uname_store);
  let message = "";

  onMount(async () => {
    if (uname) {
      await fetch(host + "/create/apple", { method: "POST" });
      connect(new URL("ws://" + host + "/connect/apple"));
    }
  });
</script>

<main>
  {#if uname}
    You're logged in as {uname}

    <input type="text" bind:value={message} />
    <button
      on:click={() =>
        sendMessage({
          sender: uname,
          room: "general",
          content: message,
          timestamp: Date.now(),
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
