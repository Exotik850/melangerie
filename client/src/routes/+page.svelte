<script lang="ts">
  import ChatRoom from "./ChatRoom.svelte";
  import { token_store } from "$lib/stores";
  import Modal from "../components/Modal.svelte";
  import toast, { Toaster } from "svelte-french-toast";
</script>

<main>
  <Toaster />
  {#if $token_store}
    <ChatRoom />
  {:else}
    <Modal />
  {/if}
</main>

<style lang="scss">
</style>
