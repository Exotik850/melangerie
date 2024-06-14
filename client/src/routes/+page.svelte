<script lang="ts">
  import ChatRoom from "./ChatRoom.svelte";
  import { tabHidden, token_store } from "$lib/stores";
  import Modal from "../components/Modal.svelte";
  import toast, { Toaster } from "svelte-french-toast";
  import { onMount } from "svelte";
  import Nav from "./Nav.svelte";

  onMount(() => {
    document.addEventListener("visibilitychange", () => {
      tabHidden.set(document.hidden);
    })
  })
</script>

<main>
  <Toaster />
  {#if $token_store}
    <Nav/>
    <ChatRoom />
  {:else}
    <Modal />
  {/if}
</main>

<style lang="scss">
  main {
    height: 100vh;
  }
</style>
