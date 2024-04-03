<script lang="ts">
  import { get } from "svelte/store";
  import Modal from "../components/Modal.svelte";
  import { onMount } from "svelte";
  import { connect, incomingMessages, sendMessage, uname_store } from "$lib/stores";

  
  let uname = get(uname_store);
  let message = "";
  $: requests = $incomingMessages.requests.reverse();

  onMount(async () => {
    if (uname) {
      await fetch("/create/apple", {method: "POST"});
      connect(new URL("ws://" + location.host + "/connect/apple"));
    }
  });
</script>

<main>
  {#if uname}
    You're logged in as {uname}

    <input type="text" bind:value={message} />
    <button on:click={() => sendMessage({ sender: uname, room: "general", content: message, timestamp: Date.now() })}>
      Send
    </button>

    <ul>
      {#each requests as request}
        <li>{request.sender} : {request.content}</li>
      {/each}
    </ul>
  {:else}
    <Modal />
  {/if}
</main>
